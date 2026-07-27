use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, HeaderValue};
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

async fn graphql_handler(
    State(schema): State<AppSchema>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    if let Ok(value) = HeaderValue::from_str(&addr.to_string()) {
        headers.insert(H_SOCKET_ADDR, value);
    }
    let req = req.into_inner().data(headers);
    schema.execute(req).await.into()
}
