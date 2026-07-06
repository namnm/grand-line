use grand_line::prelude::*;

#[grand_line_err]
enum MyErr {
    #[error("test")]
    Test,
}

#[derive(Default)]
struct Query;
#[Object]
impl Query {
    async fn my_err(&self) -> Res<i64> {
        Err(MyErr::Test.into())
    }
}

fn schema() -> GraphQLSchema<Query, EmptyMutation, EmptySubscription> {
    GraphQLSchema::build(Query, EmptyMutation, EmptySubscription).finish()
}

#[tokio::test]
async fn should_be_my_err() -> Res<()> {
    let s = schema();

    let r = s.execute("{ myErr }").await;

    let Some(err) = &r.errors.first() else {
        return TestErr::expect("response should have an error");
    };

    pretty_eq!(err.message, "test", "error message should match");

    let Some(err) = err.source.as_deref().and_then(|e| e.downcast_ref::<GrandLineErr>()) else {
        return TestErr::expect("downcast to GrandLineErr should be some");
    };

    let code = err.0.code();
    pretty_eq!(code, "Test", "error code after downcast should match");

    return Ok(());
}
