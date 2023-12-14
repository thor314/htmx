use axum::{
    extract::Query,
    response::{Html, Redirect},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::fs;

#[tokio::main]
async fn main() {
    // Define the application routes
    let app = Router::new()
        .route("/", get(serve_html))
        .route("/world", get(|| async { Redirect::permanent("/hello") }))
        .route("/hello", get(hello))
        .route("/contacts", get(contacts));

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_html() -> Html<String> {
    Html(fs::read_to_string("static/index.html").await.unwrap())
}

async fn hello() -> &'static str {
    "Hello World!"
}

/// 
#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

// Mock function to simulate contact search
fn search_contacts(query: &str) -> String {
    format!("Contacts matching '{}'", query)
}

// Mock function to get all contacts
fn get_all_contacts() -> &'static str {
    "All contacts"
}

// Handler for the `/contacts` route
async fn contacts(Query(params): Query<SearchQuery>) -> String {
    match params.q {
        Some(query) => search_contacts(&query),
        None => get_all_contacts().to_string(),
    }
}
