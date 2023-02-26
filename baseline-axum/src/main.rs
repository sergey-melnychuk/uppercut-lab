use axum::{routing::get, Router, response::IntoResponse};
use std::net::SocketAddr;

async fn handle_request() -> impl IntoResponse {
    "hello\n"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handle_request));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
