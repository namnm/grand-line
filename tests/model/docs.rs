use grand_line::prelude::*;

mod test {
    use super::*;

    #[model]
    pub struct Alias {
        pub name: String,
    }

    /// Detail one alias by id.
    #[detail(Alias)]
    fn resolver() {
    }
}
use test::*;

/// Returns the answer to everything.
#[query]
fn deep_thought() -> i64 {
    42
}

#[derive(Default, MergedObject)]
pub struct DocsQuery(AliasDetailQuery, DeepThoughtQuery);

// ---------------------------------------------------------------------------
// Generated resolvers keep their doc comments in the schema
// ---------------------------------------------------------------------------

// The whole surface of the framework is generated, so a /// comment on a
// #[query]/#[mutation] or crud resolver is the only place an api description
// can come from. The docs are carried into the generated resolver, the schema
// keeps them, and introspection is not silently empty.
#[tokio::test]
async fn resolver_doc_comments_reach_the_schema() -> Res<()> {
    let tmp = tmp_db!(Alias);
    let s = schema_q::<DocsQuery>(&tmp.db).finish();

    let sdl = s.sdl();

    pretty_eq!(
        sdl.contains("Returns the answer to everything."),
        true,
        "a #[query] fn's doc comment should become the field's description: {sdl}",
    );
    pretty_eq!(
        sdl.contains("Detail one alias by id."),
        true,
        "a crud resolver's doc comment should become the field's description: {sdl}",
    );
    pretty_eq!(
        sdl.contains("\"\"\""),
        true,
        "the docs should be carried as sdl descriptions: {sdl}",
    );

    tmp.drop().await
}
