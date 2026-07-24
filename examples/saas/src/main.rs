mod am_ctx;
mod auth;
mod authz;
mod authz_impl;
mod err;
mod models;

use grand_line::prelude::*;

pub use auth::*;
pub use authz::*;
pub use authz_impl::*;
pub use err::*;
pub use models::*;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::post;
use axum::{Router, serve};
use tokio::net::TcpListener;

type AppSchema = GraphQLSchema<Query, Mutation, EmptySubscription>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let db = db().await?;
    let schema = schema(&db);

    let app = Router::new().route("/api/graphql", post(graphql_handler)).with_state(schema);

    let port = 4000;
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(addr).await?;

    println!("listening on port {port}");
    serve(listener, app).await?;

    Ok(())
}

/// Injects the raw axum HeaderMap into the GraphQL request's data so resolvers
/// can read headers (org/role id, session token, ...) via ctx.get_header(...).
/// async_graphql_axum's plain GraphQL service doesn't do this on its own since
/// it has no way to know which headers a given app's resolvers care about, so
/// this app writes its own handler instead of using GraphQL::new(schema) directly.
async fn graphql_handler(State(schema): State<AppSchema>, headers: HeaderMap, req: GraphQLRequest) -> GraphQLResponse {
    let req = req.into_inner().data(headers);
    schema.execute(req).await.into()
}

// ----------------------------------------------------------------------------
// init schema

grand_line::include_generated_schema! {}

fn schema(db: &DatabaseConnection) -> GraphQLSchema<Query, Mutation, EmptySubscription> {
    let org_impl = Org::authz_default_impl();
    let role_impl: Box<dyn AuthzRoleImpl> = Box::new(SaasRoleImpl);
    let user_impl: Box<dyn AuthzCurrentUserImpl> = Box::new(SaasCurrentUserImpl);

    GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
        .extension(GrandLineExtension)
        .data(Arc::new(db.clone()))
        .data(org_impl)
        .data(role_impl)
        .data(user_impl)
        .finish()
}

// ----------------------------------------------------------------------------
// init db

async fn db() -> Result<DatabaseConnection, Box<dyn Error + Send + Sync>> {
    let db = Database::connect("sqlite::memory:").await?;

    let backend = db.get_database_backend();
    let schema = DbSchema::new(backend);
    for stmt in [
        schema.create_table_from_entity(User),
        schema.create_table_from_entity(Org),
        schema.create_table_from_entity(Role),
        schema.create_table_from_entity(UserInRole),
        schema.create_table_from_entity(LoginSession),
        schema.create_table_from_entity(Otp),
        schema.create_table_from_entity(Impersonation),
    ] {
        db.execute(backend.build(&stmt)).await?;
    }

    seed(&db).await?;

    Ok(db)
}

/// Seeds a bootstrap org, a system-realm role with a wildcard col_policy, and a user
/// assigned to it, so the app has an initial actor able to hit org-realm-authz endpoints
/// (system passes into org via SaasRoleImpl) without a chicken-and-egg role_create call.
/// Real deployments would replace this with an out-of-band ops/migration step.
async fn seed(db: &DatabaseConnection) -> Result<(), Box<dyn Error + Send + Sync>> {
    let root = am_create!(User {
        email: "root@example.com".to_owned(),
        password_hashed: rand_utils::password_hash("Bootstrap-R00t!")?,
    })
    .exec_without_ctx(db)
    .await?;

    let org = am_create!(Org {
        name: "Acme Inc".to_owned(),
    })
    .exec_without_ctx(db)
    .await?;

    let wildcard = json!({
        "*": {
            "inputs": { "allow": true, "children": { "**": { "allow": true, "children": null } } },
            "output": { "allow": true, "children": { "**": { "allow": true, "children": null } } },
        },
    });
    let role = am_create!(Role {
        name: "Bootstrap Admin".to_owned(),
        realm: "system".to_owned(),
        col_policy: wildcard,
        row_policy: json!({}),
        org_id: None,
    })
    .exec_without_ctx(db)
    .await?;

    am_create!(UserInRole {
        user_id: root.id.clone(),
        role_id: role.id.clone(),
        org_id: None,
    })
    .exec_without_ctx(db)
    .await?;

    println!("seeded bootstrap user root@example.com / Bootstrap-R00t! (user_id={})", root.id);
    println!("seeded bootstrap org {} (org_id={})", org.name, org.id);
    println!("seeded bootstrap role {} (role_id={})", role.name, role.id);

    Ok(())
}
