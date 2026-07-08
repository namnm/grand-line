pub use grand_line::prelude::*;

#[grand_line_err]
enum MyErr {
    #[error("should be exposed to the client")]
    #[client]
    Client,
    #[error("server error should not be exposed to the client")]
    Server,
}

#[derive(Default)]
struct Query;
#[Object]
impl Query {
    async fn client(&self) -> Res<i64> {
        Err(MyErr::Client.into())
    }
    async fn server(&self) -> Res<i64> {
        Err(MyErr::Server.into())
    }
    async fn std(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Err("any other error such as std should not be exposed to the client".into())
    }
}

#[tokio::test]
async fn should_only_expose_client_errors() -> Res<()> {
    let tmp = tmp_db().await?;
    let s = schema_q::<Query>(&tmp.db).finish();

    check(&s, "{ client }", MyErr::Client).await;
    check(&s, "{ server }", CoreGraphQLErr::InternalServer).await;
    check(&s, "{ std }", CoreGraphQLErr::InternalServer).await;

    tmp.drop().await
}

async fn check<T>(s: &GraphQLSchema<Query, EmptyMutation, EmptySubscription>, req: &str, e: T)
where
    T: GrandLineErrImpl,
{
    let r = s.execute(req).await;
    check_err(&r, &e);
}
