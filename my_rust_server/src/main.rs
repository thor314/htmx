use std::{collections::HashMap, sync::Arc};

use axum::{
  extract::{Path, Query, State},
  http::{self, Response, StatusCode},
  response::{Html, IntoResponse, Redirect},
  routing::get,
  Extension, Form, Router,
};
use contact::{Contact, Contacts, DatabaseConnection};
use http::header::SET_COOKIE;
use serde::Deserialize;
use tera::Tera;
use tokio::sync::Mutex;

mod contact;

type ContactsState = State<Arc<Mutex<Contacts>>>;

#[tokio::main]
async fn main() {
  // Define the application routes
  let tera = Arc::new(tera::Tera::new("templates/**/*").expect("failure parsing templates"));
  let db = Arc::new(Mutex::new(Contacts::load_db()));

  let app = Router::new()
    .route("/", get(get_index))
    .route("/world", get(|| async { Redirect::permanent("/hello") }))
    .route("/hello", get(get_hello))
    .route("/contacts", get(get_contacts))
    // .route("/contacts/new", get(get_new_contact).post(post_new_contact))
    .route("/contacts/:contact_id", get(get_contacts_view))
    .layer(Extension(tera))
    // .layer(Extension(contacts));
    .with_state(db);

  // Run the server
  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}

async fn get_hello() -> &'static str { "Hello World!" }

async fn get_index(tera: Extension<Arc<Tera>>) -> Html<String> {
  let context = tera::Context::new();
  let rendered = tera.render("index.html", &context).expect("Failed to render template");
  Html(rendered)
}

#[derive(Deserialize)]
struct SearchQuery {
  q: Option<String>,
}

/// Handler for the `/contacts` route.
/// Allow the user to search for a particular contact.
async fn get_contacts(
  Query(query): Query<SearchQuery>,
  State(contacts): ContactsState,
  tera: Extension<Arc<Tera>>,
) -> Html<String> {
  let mut context = tera::Context::new();
  let contacts = contacts.lock().await;

  match query.q {
    Some(ref search) => {
      let contacts = contacts.search(search);
      context.insert("q", search);
      context.insert("contacts", &contacts);
    },
    None => context.insert("contacts", &*contacts),
  };

  let rendered = tera.render("contacts.html", &context).expect("Failed to render template");

  Html(rendered)
}

/// look for a contact with some contact id.
async fn get_contacts_view(
  Path(contact_id): Path<usize>,
  tera: Extension<Arc<Tera>>,
  State(contacts): ContactsState,
) -> Html<String> {
  dbg!(contact_id);
  let contacts = contacts.lock().await;
  let mut context = tera::Context::new();
  let contact = contacts.get(contact_id).unwrap();
  context.insert("contact", &contact);

  let rendered = tera.render("show.html", &context).expect("Failed to render template");

  Html(rendered)
}

async fn get_new_contact(Extension(tera): Extension<Arc<Tera>>) -> Html<String> {
  // seems dumb to call this "errors"
  let _cookies = http::header::HeaderValue::from_static("flash=; Path=/; HttpOnly");
  let mut errors = HashMap::new();

  errors.insert("phone", "Phone number is required.");
  errors.insert("last", "Phone number is required.");
  errors.insert("first", "Phone number is required.");
  errors.insert("id", "Phone number is required.");
  errors.insert("email", "Phone number is required.");

  let mut context = tera::Context::new();
  context.insert("errors", &errors);

  let rendered = tera.render("new.html", &context).expect("Failed to render template");
  Html(rendered)
}

async fn post_new_contact(
  Form(contact): Form<Contact>,
  DatabaseConnection(contacts): DatabaseConnection,
) -> Result<impl IntoResponse, Html<String>> {
  dbg!(&contact);
  let mut contacts = contacts.lock().await;

  contacts.insert(contact);
  contacts.save_db();

  // create a cookie and redirect to contacts
  let response = Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(
            SET_COOKIE,
            format!("flash=Created New Contact!; Path=/; HttpOnly"),
        )
        .header(http::header::LOCATION, "/contacts") // will redirect back to contacts
        .body(axum::body::Body::from("Redirecting..."))
        .unwrap();

  Ok(response)
}
