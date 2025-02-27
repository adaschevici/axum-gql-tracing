use crate::model::QueryRoot;
use crate::observability::metrics::{create_prometheus_recorder, track_metrics};
use crate::observability::tracing::create_tracer_from_env;
use crate::routes::{graphql_handler, graphql_playground, health};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use axum::{Router, body::Body, extract::Extension, middleware, routing::get};
use dotenv::dotenv;
use std::future::ready;
use tokio::time::Duration;
use tokio::{select, signal};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod model;
mod observability;
mod routes;
// async fn shutdown_signal() {
//     // (1)
//     let ctrl_c = async {
//         signal::ctrl_c()
//             .await
//             .expect("failed to install Ctrl+C handler");
//     };
//
//     #[cfg(unix)]
//     let terminate = async {
//         signal::unix::signal(signal::unix::SignalKind::terminate())
//             .expect("failed to install signal handler")
//             .recv()
//             .await;
//     };
//
//     #[cfg(not(unix))]
//     let terminate = std::future::pending::<()>();
//
//     select! {
//         _ = ctrl_c => {
//             println!("Ctrl-C received, shutting down...");
//         },
//         _ = terminate => {},
//     }
//
//     println!("Shutdown signal received, cleaning up...");
//
//     opentelemetry::global::shutdown_tracer_provider();
// }
#[tokio::main] // (1)
async fn main() {
    dotenv().ok();
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let registry = Registry::default().with(tracing_subscriber::fmt::layer().pretty()); // (2)

    match create_tracer_from_env() {
        // (3)
        Some(tracer) => registry
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .try_init()
            .expect("Failed to register tracer with registry"),
        None => registry
            .try_init()
            .expect("Failed to register tracer with registry"),
    }

    let prometheus_recorder = create_prometheus_recorder(); // (1)
    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route("/health", get(health))
        .route("/metrics", get(move || ready(prometheus_recorder.render()))) // (1)
        .route_layer(middleware::from_fn(track_metrics::<Body>)) // (2)
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
        ))
        .layer(Extension(schema)); // (2)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    println!("Cleaning up...");
    opentelemetry::global::shutdown_tracer_provider();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
