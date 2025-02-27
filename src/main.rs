use crate::model::QueryRoot;
use crate::routes::{graphql_handler, graphql_playground, health};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use axum::{Router, extract::Extension, routing::get};

mod model;
mod routes;

#[tokio::main] // (1)
async fn main() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route("/health", get(health))
        .layer(Extension(schema)); // (2)

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
