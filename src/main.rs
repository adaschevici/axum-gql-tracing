use crate::model::QueryRoot;
use crate::observability::metrics::{create_prometheus_recorder, track_metrics};
use crate::routes::{graphql_handler, graphql_playground, health};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use axum::{Router, body::Body, extract::Extension, middleware, routing::get};
use std::future::ready;

mod model;
mod observability;
mod routes;

#[tokio::main] // (1)
async fn main() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let prometheus_recorder = create_prometheus_recorder(); // (1)
    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route("/health", get(health))
        .route("/metrics", get(move || ready(prometheus_recorder.render()))) // (1)
        .route_layer(middleware::from_fn(track_metrics::<Body>)) // (2)
        .layer(Extension(schema)); // (2)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
