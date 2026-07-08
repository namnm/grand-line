use grand_line::prelude::*;
use serde_json::to_string as json;

// create a sea orm model and graphql object
// id, created_at, updated_at, deleted_at... are inserted automatically
#[model]
pub struct Todo {
    pub content: String,
    pub done: bool,
}

// search Todo with filter, sort, pagination from client
// variables are generated automatically
#[search(Todo)]
fn resolver() {
    let f = json(&filter)?;
    let o = json(&order_by)?;
    let p = json(&page)?;
    println!("todoSearch filter={f} order_by={o} page={p}");
    (None, None)
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
    (Some(extra_filter), Some(default_order_by))
}

// count Todo with filter from client
#[count(Todo)]
fn resolver() {
    let f = json(&filter)?;
    println!("todoCount filter={f}");
    None
}

// get detail of a Todo by id
#[detail(Todo)]
fn resolver() {
    println!("todoDetail id={id}");
}

// create a new Todo
#[gql_input]
pub struct TodoCreate {
    pub content: String,
}
#[create(Todo)]
fn resolver() {
    let d = json(&data)?;
    println!("todoCreate data={d}");
    am_create!(Todo {
        content: data.content,
    })
}

// update a Todo content
#[gql_input]
pub struct TodoUpdate {
    pub content: String,
}
#[update(Todo)]
fn resolver() {
    let d = json(&data)?;
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
    Todo::soft_delete_many()?
        .filter(f.clone().into_condition())
        .exec(tx)
        .await?;
    f.gql_select_id().all(tx).await?
}

// ----------------------------------------------------------------------------
// main axum listener

use async_graphql_axum::GraphQL;
use axum::{Router, routing::get_service, serve};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let db = db().await?;
    let schema = schema(&db);

    let svc = GraphQL::new(schema);
    let router = get_service(svc.clone()).post_service(svc);
    let app = Router::new().route("/api/graphql", router);

    let port = 4000;
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(addr).await?;

    println!("listening on port {port}");
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

async fn db() -> Result<DatabaseConnection, Box<dyn Error + Send + Sync>> {
    let db = Database::connect("sqlite::memory:").await?;

    let backend = db.get_database_backend();
    let schema = DbSchema::new(backend);
    let stmt = schema.create_table_from_entity(Todo);
    db.execute(backend.build(&stmt)).await?;

    Todo::insert_many(vec![
        am_create!(Todo {
            content: "2023 good bye",
            done: true,
        })
        .into_active_model(),
        am_create!(Todo {
            content: "2023 great",
            done: true,
        })
        .into_active_model(),
        am_create!(Todo {
            content: "2024 hello",
            done: false,
        })
        .into_active_model(),
        am_create!(Todo {
            content: "2024 awesome",
            done: false,
        })
        .into_active_model(),
    ])
    .exec(&db)
    .await?;

    Ok(db)
}
