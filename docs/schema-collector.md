# Schema collector

Each resolver macro generates a named struct (`TodoSearchQuery`, `TodoCreateMutation`, etc.). Normally you must list all of them manually in a `MergedObject`:

```rs
// Manual - must add each resolver type by hand
#[derive(Default, MergedObject)]
struct Query(TodoSearchQuery, TodoCountQuery, TodoDetailQuery, TodoCountDoneQuery);

#[derive(Default, MergedObject)]
struct Mutation(
    TodoCreateMutation,
    TodoUpdateMutation,
    TodoDeleteMutation,
    TodoDeleteDoneMutation,
);
```

`grand_line_build` eliminates this by scanning source files at build time and auto-generating `Query` and `Mutation`. It works across crates - any source directory can be included.

Add it as a build dependency:

```toml
[build-dependencies]
grand_line_build = { path = "../../packages/grand_line_build" }
```

Create or edit `build.rs` at the crate root:

```rs
```

This scans `src/` of the current crate. Then include the generated file in your crate root:

```rs
grand_line::include_generated_schema! {}

fn schema(db: &DatabaseConnection) -> GraphQLSchema<Query, Mutation, EmptySubscription> {
    GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
        .extension(GrandLineExtension)
        .data(Arc::new(db.clone()))
        .finish()
}
```

For more control - multiple source directories and extra merged types (e.g. hand-written resolvers that live outside the scanned directories):

```rs
fn main() {
    grand_line_build::SchemaBuilder::new()
        .scan("src")
        .scan("../other_crate/src")
        .extra_query("SomeExtraQuery")
        .extra_mutation("SomeExtraMutation")
        .generate();
}
```

The generated `Query` and `Mutation` match the names produced by the resolver macros exactly (same naming convention). `rerun-if-changed` directives are emitted automatically for each scanned directory.
