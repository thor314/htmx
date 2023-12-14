use axum::{
    extract::Query,
    response::{Html, Redirect},
    routing::get,
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tera::Tera;
// use tokio::fs;

#[tokio::main]
async fn main() {
    // Define the application routes
    let tera = Arc::new(tera::Tera::new("templates/**/*").unwrap());
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/world", get(|| async { Redirect::permanent("/hello") }))
        .route("/hello", get(hello))
        .route("/contacts", get(contact_handler))
        // get(move |query: Query<SearchQuery>| contact_handler(query, tera.clone())),
        .layer(Extension(tera)); // Add the Tera instance as a shared state
                                 // ;

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> &'static str {
    "Hello World!"
}

async fn index_handler(tera: Extension<Arc<Tera>>) -> Html<String> {
    let context = tera::Context::new();
    let rendered = tera
        .render("index.html", &context)
        .expect("Failed to render template");
    Html(rendered)
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

// Handler for the `/contacts` route
// async fn contact_handler(Query(params): Query<SearchQuery>, tera: Arc<Tera>) -> Html<String> {
async fn contact_handler(
    Query(query): Query<SearchQuery>,
    tera: Extension<Arc<Tera>>,
) -> Html<String> {
    let mut context = tera::Context::new();
    match query.q {
        Some(ref search) => {
            let contact = search_contacts(search);
            context.insert("q", search);
            context.insert("contacts", &contact);
        }
        None => context.insert("contacts", &get_all_contacts()),
    };

    let rendered = tera
        .render("contacts.html", &context)
        .expect("Failed to render template");

    Html(rendered)
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Contact {
    id: String,
    first: String,
    last: String,
    email: String,
    phone: String,
    // name: String,
    // other fields...
}

// Mock function to simulate searching contacts
fn search_contacts(_query: &str) -> Vec<Contact> {
    vec![Contact {
        id: "Joe".to_string(),
        first: "Joe".to_string(),
        last: "Doe".to_string(),
        email: "mail doe".to_string(),
        phone: "phone doe".to_string(),
    }]
}

// Mock function to simulate retrieving all contacts
fn get_all_contacts() -> Vec<Contact> {
    // In a real application, retrieve all contacts from the database or data source here
    vec![
        Contact {
            id: "Joe".to_string(),
            first: "Joe".to_string(),
            last: "Doe".to_string(),
            email: "mail doe".to_string(),
            phone: "phone doe".to_string(),
        },
        Contact {
            id: "Joe".to_string(),
            first: "Jane".to_string(),
            last: "Doe".to_string(),
            email: "mail".to_string(),
            phone: "phone".to_string(),
        },
        Contact {
            id: "Joe".to_string(),
            first: "John".to_string(),
            last: "Doe".to_string(),
            email: "mail".to_string(),
            phone: "phone".to_string(),
        },
    ]
}
