use crate::routes::health;
use axum::{Router, routing::get};

mod routes;

#[tokio::main] // (1)
async fn main() {
    let app = Router::new().route("/health", get(health)); // (2)

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
