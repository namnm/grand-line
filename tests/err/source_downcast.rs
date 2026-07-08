pub use grand_line::prelude::*;

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
async fn should_be_my_err() {
    let s = schema();

    let r = s.execute("{ myErr }").await;
    assert!(r.errors.len() == 1, "response should have an error");

    let Some(err) = &r.errors.first() else {
        return;
    };
    pretty_eq!(err.message, "test", "error message should match");

    let err = err.source.as_deref().and_then(|e| e.downcast_ref::<GrandLineErr>());
    if let Some(err) = err {
        let code = err.0.code();
        pretty_eq!(code, "Test", "error code after downcast should match");
    } else {
        assert!(false, "downcast to GrandLineErr should be some");
    }
}
