use axum::{
    routing::get,
    Router,
    response::Html,
};
use std::net::SocketAddr;
use tokio::fs;

#[tokio::main]
async fn main() {
    // Define the application routes
    let app = Router::new().route("/", get(serve_html));

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn serve_html() -> Html<String> {
    Html(fs::read_to_string("static/index.html").await.unwrap())
}
