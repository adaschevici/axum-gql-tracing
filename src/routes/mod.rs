use crate::model::ServiceSchema;
use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Json,
    extract::Extension,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use serde::Serialize;

#[derive(Serialize)]
struct Health {
    healthy: bool,
}

pub(crate) async fn health() -> impl IntoResponse {
    let health = Health { healthy: true };

    (StatusCode::OK, Json(health))
}

pub(crate) async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        // (1)
        GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
    ))
}

pub(crate) async fn graphql_handler(
    Extension(schema): Extension<ServiceSchema>, // (2)
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into() // (3)
}
