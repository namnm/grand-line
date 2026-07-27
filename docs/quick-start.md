# Quick start

```rs
use grand_line::prelude::*;

#[model]
pub struct Todo {
    pub content: String,
    pub done: bool,
}

#[search(Todo)]
fn resolver() {
}

#[gql_input]
pub struct TodoCreate {
    pub content: String,
}
#[create(Todo)]
fn resolver() {
    am_create!(Todo {
        content: data.content,
    })
}
```

<p align="center">
  <img src="https://github.com/nongdan-dev/grand-line/blob/master/.md/altair.jpg?raw=true" alt="Altair screenshot"/>
</p>

That produces a `todoSearch` query with filter/sort/pagination, and a `todoCreate` mutation - all type-safe, all wired to the database.

See [Schema collector](schema-collector.md) for how `Query`/`Mutation` get built from resolvers like these, and the [Simple Todo example](https://github.com/nongdan-dev/grand-line/blob/master/examples/simple_todo/src/lib.rs) for a complete, runnable CRUD API.
