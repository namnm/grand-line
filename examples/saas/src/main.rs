use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::middleware;
use axum::routing::post;
use axum::{Router, serve};
use core::net::SocketAddr;
use grand_line_examples_saas::prelude::*;
use grand_line_examples_saas::{AppSchema, db, schema};
use std::env;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let db = db().await?;
    let schema = schema(&db).finish();

    let app = Router::new()
        .route("/api/graphql", post(graphql_handler))
        .with_state(schema)
        // Fills x-socket-addr from the real connection, the source get_ip reads
        // by default, so the safe configuration cannot be forgotten.
        .layer(middleware::from_fn(socket_addr_layer))
        .layer(CorsLayer::permissive())
        .into_make_service_with_connect_info::<SocketAddr>();

    let hostname = env::var("HOSTNAME").unwrap_or_else(|_| "0.0.0.0".to_owned());
    let port = env::var("PORT").unwrap_or_else(|_| "4000".to_owned());
    let addr = format!("{hostname}:{port}");
    let listener = TcpListener::bind(addr).await?;

    println!("listening on {hostname}:{port}");
    serve(listener, app).await?;

    Ok(())
}

/// The HeaderMap still has to reach the graphql context, HttpContext reads it
/// from the request data (see HttpAxumContext::get_headers), so every ctx header
/// helper, bearer token and cookie included, depends on this. socket_addr_layer
/// only fills x-socket-addr in the request headers, it cannot inject the map.
async fn graphql_handler(State(schema): State<AppSchema>, headers: HeaderMap, req: GraphQLRequest) -> GraphQLResponse {
    let req = req.into_inner().data(headers);
    schema.execute(req).await.into()
}
