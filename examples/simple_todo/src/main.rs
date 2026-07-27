use async_graphql_axum::GraphQL;
use axum::{
    Router,
    routing::{get, get_service},
    serve,
};
use grand_line_examples_simple_todo::prelude::*;
use grand_line_examples_simple_todo::{db, hello_rest, schema};
use std::env;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let db = db().await?;
    let schema = schema(&db).finish();

    let svc = GraphQL::new(schema);
    let gql = get_service(svc.clone()).post_service(svc);

    let app = Router::new()
        .route("/api/graphql", gql)
        .route("/api/fetch", get(hello_rest))
        .layer(CorsLayer::permissive());

    let hostname = env::var("HOSTNAME").unwrap_or_else(|_| "0.0.0.0".to_owned());
    let port = env::var("PORT").unwrap_or_else(|_| "4000".to_owned());
    let addr = format!("{hostname}:{port}");
    let listener = TcpListener::bind(addr).await?;

    println!("listening on {hostname}:{port}");
    serve(listener, app).await?;

    Ok(())
}
