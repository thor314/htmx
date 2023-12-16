use std::sync::Arc;

use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{Html, IntoResponse, Response},
  routing::{delete, get},
  Extension, Form, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tera::Tera;
use tokio::sync::broadcast::{channel, Sender};
use tower::ServiceBuilder;

pub type TodosStream = Sender<TodoUpdate>;

// src/main.rs
#[derive(Clone, Serialize, Debug)]
enum MutationKind {
  Create,
  Delete,
}

#[derive(Clone, Serialize, Debug)]
pub struct TodoUpdate {
  mutation_kind: MutationKind,
  id:            i32,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
struct Todo {
  id:          i32,
  description: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
struct TodoNew {
  description: String,
}

#[derive(Clone)]
struct AppState {
  db: PgPool,
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] db: PgPool) -> shuttle_axum::ShuttleAxum {
  sqlx::migrate!().run(&db).await.expect("Looks like something went wrong with migrations :(");

  let (tx, _rx) = channel::<TodoUpdate>(10);
  let state = AppState { db };
  let tera = Arc::new(tera::Tera::new("templates/**/*").expect("failure parsing templates"));
  let router = init_router(state, tx, tera).expect("Failed to init router");

  Ok(router.into())
}

fn init_router(
  state: AppState,
  tx: TodosStream,
  tera: Arc<tera::Tera>,
) -> Result<Router<()>, axum::Error> {
  let router = Router::new()
    .route("/", get(home))
    .route("/stream", get(stream))
    .route("/styles.css", get(styles))
    .route("/todos", get(fetch_todos).post(create_todo))
    .route("/todos/:id", delete(delete_todo))
    .route("/todos/stream", get(stream::handle_stream))
    .with_state(state)
    .layer(
      ServiceBuilder::new()
                // .layer(tower_http::trace::TraceLayer::new_for_http())
                .layer(tower_http::compression::CompressionLayer::new())
                // .layer(tower_http::add_extension::AddExtensionLayer::new(tera))
                .layer(Extension(tera))
                .layer(Extension(tx))
                .into_inner(),
    );

  Ok(router)
}

type TeraExt = Extension<Arc<Tera>>;

async fn home(tera: TeraExt) -> Html<String> {
  let context = tera::Context::new();
  let rendered = tera.render("base.html", &context).expect("Failed to render template");
  Html(rendered)
}

// check stream?
async fn stream(tera: TeraExt) -> impl IntoResponse {
  let context = tera::Context::new();
  let rendered = tera.render("stream.html", &context).expect("Failed to render template");
  Html(rendered)
}

async fn fetch_todos(State(state): State<AppState>, Extension(tera): TeraExt) -> Html<String> {
  let mut context = tera::Context::new();
  let todos = sqlx::query_as::<_, Todo>("SELECT * FROM TODOS").fetch_all(&state.db).await.unwrap();

  context.insert("todos", &todos);
  let rendered = tera.render("todos.html", &context).expect("Failed to render template");
  Html(rendered)
}

pub async fn styles() -> impl IntoResponse {
  Response::builder()
    .status(StatusCode::OK)
    .header("Content-Type", "text/css")
    .body(include_str!("../static/styles.css").to_owned())
    .unwrap()
}

async fn create_todo(
  State(state): State<AppState>,
  Extension(tx): Extension<TodosStream>,
  Extension(tera): TeraExt,
  Form(form): Form<TodoNew>,
) -> Html<String> {
  let mut context = tera::Context::new();
  let todo = sqlx::query_as::<_, Todo>(
    "INSERT INTO TODOS (description) VALUES ($1) RETURNING id, description",
  )
  .bind(form.description)
  .fetch_one(&state.db)
  .await
  .unwrap();

  if let Err(e) =
    tx.send(TodoUpdate { mutation_kind: MutationKind::Create, id: todo.id })
  {
    eprintln!(
      "Tried to send log of record with ID {} created but something went wrong: {e}",
      todo.id
    );
  }

  context.insert("todo", &todo);
  let rendered = tera.render("todo.html", &context).expect("Failed to render template");
  Html(rendered)
}

async fn delete_todo(
  State(state): State<AppState>,
  Path(id): Path<i32>,
  Extension(tx): Extension<TodosStream>,
) -> impl IntoResponse {
  sqlx::query("DELETE FROM TODOS WHERE ID = $1").bind(id).execute(&state.db).await.unwrap();

  if let Err(e) = tx.send(TodoUpdate { mutation_kind: MutationKind::Delete, id }) {
    eprintln!("Tried to send log of record with ID {id} created but something went wrong: {e}");
  }

  StatusCode::OK
}

mod stream {
  use std::{convert::Infallible, time::Duration};

  use axum::response::{sse::Event, Sse};
  use serde_json::json;
  use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt as _};

  use super::*;
  pub async fn handle_stream(
    Extension(tx): Extension<TodosStream>,
  ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = tx.subscribe();

    let stream = BroadcastStream::new(rx);

    Sse::new(
      stream
        .map(|msg| {
          let msg = msg.unwrap();
          let json = format!("<div>{}</div>", json!(msg));
          Event::default().data(json)
        })
        .map(Ok),
    )
    .keep_alive(
      axum::response::sse::KeepAlive::new()
        .interval(Duration::from_secs(600))
        .text("keep-alive-text"),
    )
  }
}
