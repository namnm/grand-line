use grand_line::prelude::*;

use async_graphql_axum::GraphQL;
use axum::{
    Json, Router,
    routing::{get, get_service},
    serve,
};
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

// ---------------------------------------------------------------------------
// Todo model and GraphQL resolvers
// ---------------------------------------------------------------------------

// create a sea orm model and graphql object
// id, created_at, updated_at, deleted_at... are inserted automatically

/// A todo item, persisted in the database and exposed over GraphQL.
#[model]
pub struct Todo {
    pub content: String,
    pub done: bool,
}

// search Todo with filter, sort, pagination from client
// variables are generated automatically
#[search(Todo)]
fn resolver() {
    let f = json_string(&filter)?;
    let o = json_string(&order_by)?;
    let p = json_string(&page)?;
    println!("todoSearch filter={f} order_by={o} page={p}");
}

// we can also have a custom name
// with extra filter, or default sort in the resolver as well
// the extra will be combined as and condition with the value from client
#[search(Todo)]
fn todo_search_2024() {
    let extra_filter = filter!(Todo {
        content_starts_with: "2024",
    });
    let default_order_by = order_by!(Todo [DoneAsc, ContentAsc]);
    (extra_filter, default_order_by).into()
}

// count Todo with filter from client
#[count(Todo)]
fn resolver() {
    let f = json_string(&filter)?;
    println!("todoCount filter={f}");
}

// get detail of a Todo by id
#[detail(Todo)]
fn resolver() {
    println!("todoDetail id={id}");
}

// create a new Todo

/// Input payload for creating a new Todo.
#[gql_input]
pub struct TodoCreate {
    pub content: String,
}
#[create(Todo)]
fn resolver() {
    let d = json_string(&data)?;
    println!("todoCreate data={d}");
    am_create!(Todo {
        content: data.content,
    })
}

// update a Todo content

/// Input payload for updating a Todo's content.
#[gql_input]
pub struct TodoUpdate {
    pub content: String,
}
#[update(Todo)]
fn resolver() {
    let d = json_string(&data)?;
    println!("todoUpdate id={id} data={d}");
    Todo::find_by_id(&id).exists_or_404(tx).await?;
    am_update!(Todo {
        id: id.clone(),
        content: data.content,
    })
}

// toggle a Todo done using update macro
// with custom resolver name and inputs
#[update(Todo, resolver_inputs)]
fn todo_toggle_done(id: String) {
    println!("todoToggleDone id={id}");
    let todo = Todo::find_by_id(&id).one_or_404(tx).await?;
    am_update!(Todo {
        id: id.clone(),
        done: !todo.done,
    })
}

// delete a Todo by id
#[delete(Todo)]
fn resolver() {
    println!("todoDelete id={id}");
    Todo::find_by_id(&id).exists_or_404(tx).await?;
}

// manual query: count number of all done Todo
#[query]
fn todo_count_done() -> u64 {
    let f = filter!(Todo {
        done: true,
    });
    f.into_select().count(tx).await?
}

// manual mutation: soft delete all done Todo
#[mutation]
fn todo_delete_done() -> Vec<TodoGql> {
    let f = filter!(Todo {
        done: true,
    });
    Todo::soft_delete_many()?.filter(f.clone()).exec(tx).await?;
    f.gql_select_id().all(tx).await?
}

// ----------------------------------------------------------------------------
// test hello world for rntwsc
// ----------------------------------------------------------------------------

/// Response body for the REST hello endpoint.
#[derive(Serialize)]
pub struct HelloRest {
    pub message: String,
    pub timestamp: i64,
}

/// REST endpoint returning a hello message after a random delay, mirrors the nodejs /api/fetch example.
async fn hello_rest() -> Json<HelloRest> {
    random_delay().await;
    Json(HelloRest {
        message: "hello".to_owned(),
        timestamp: now_millis(),
    })
}

/// Response body for the hello GraphQL query.
#[derive(SimpleObject)]
pub struct HelloGql {
    pub message: String,
    pub timestamp: i64,
}

// manual query: hello world with a random delay, mirrors the nodejs /api/graphql example
#[query]
fn hello() -> HelloGql {
    random_delay().await;
    HelloGql {
        message: "hello from graphql".to_owned(),
        timestamp: now_millis(),
    }
}

/// Sleeps for a pseudo-random duration between 0 and 1 second.
async fn random_delay() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    sleep(Duration::from_millis((nanos % 1000) as u64)).await;
}

/// Current unix time in milliseconds.
fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ----------------------------------------------------------------------------
// main axum listener
// ----------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let db = db().await?;
    let schema = schema(&db);

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

// ----------------------------------------------------------------------------
// init schema

grand_line::include_generated_schema! {}

fn schema(db: &DatabaseConnection) -> GraphQLSchema<Query, Mutation, EmptySubscription> {
    GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
        .extension(GrandLineExtension)
        .data(Arc::new(db.clone()))
        .finish()
}

// ----------------------------------------------------------------------------
// init db
// ----------------------------------------------------------------------------

async fn db() -> Result<DatabaseConnection, Box<dyn Error + Send + Sync>> {
    let db = Database::connect("sqlite://db.sqlite3?mode=rwc").await?;

    let backend = db.get_database_backend();
    let schema = DbSchema::new(backend);
    let stmt = schema.create_table_from_entity(Todo);
    db.execute(backend.build(&stmt)).await?;

    Todo::insert_many(vec![
        am_create!(Todo {
            content: "2023 good bye",
            done: true,
        })
        .into_am_without_ctx(),
        am_create!(Todo {
            content: "2023 great",
            done: true,
        })
        .into_am_without_ctx(),
        am_create!(Todo {
            content: "2024 hello",
            done: false,
        })
        .into_am_without_ctx(),
        am_create!(Todo {
            content: "2024 awesome",
            done: false,
        })
        .into_am_without_ctx(),
    ])
    .exec(&db)
    .await?;

    Ok(db)
}
