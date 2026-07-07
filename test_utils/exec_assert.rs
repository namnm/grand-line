use crate::prelude::*;

/// Execute a query/mutation and assert the response data equals expected.
pub async fn exec_assert<Q, M, S>(s: &GraphQLSchema<Q, M, S>, q: &str, v: Option<GraphQLValue>, expected: &GraphQLValue)
where
    Q: ObjectType + Default + 'static,
    M: ObjectType + Default + 'static,
    S: SubscriptionType + 'static,
{
    let res = exec_assert_ok(s, q, v).await;
    pretty_eq!(res.data, expected.clone(), "response data should match");
}

/// Like exec_assert, but supplies id as the sole GraphQL variable.
pub async fn exec_assert_id<Q, M, S>(s: &GraphQLSchema<Q, M, S>, q: &str, id: &str, expected: &GraphQLValue)
where
    Q: ObjectType + Default + 'static,
    M: ObjectType + Default + 'static,
    S: SubscriptionType + 'static,
{
    let v = value!({
        "id": id,
    });
    exec_assert(s, q, Some(v), expected).await;
}

/// Execute a query/mutation, assert there are no errors, and return the response.
pub async fn exec_assert_ok<Q, M, S>(s: &GraphQLSchema<Q, M, S>, q: &str, v: Option<GraphQLValue>) -> Response
where
    Q: ObjectType + Default + 'static,
    M: ObjectType + Default + 'static,
    S: SubscriptionType + 'static,
{
    let res = exec(s, q, v).await;
    pretty_eq!(res.errors.is_empty(), true, "response errors should be empty");
    res
}

/// Execute a query/mutation and assert the response's first error matches e.
pub async fn exec_assert_err<Q, M, S, E>(s: &GraphQLSchema<Q, M, S>, q: &str, v: Option<GraphQLValue>, e: &E) -> Res<()>
where
    Q: ObjectType + Default + 'static,
    M: ObjectType + Default + 'static,
    S: SubscriptionType + 'static,
    E: GrandLineErrImpl,
{
    let res = exec(s, q, v).await;
    check_err(&res, e)
}

/// Like exec_assert_err, but supplies id as the sole GraphQL variable.
pub async fn exec_assert_err_id<Q, M, S, E>(s: &GraphQLSchema<Q, M, S>, q: &str, id: &str, e: &E) -> Res<()>
where
    Q: ObjectType + Default + 'static,
    M: ObjectType + Default + 'static,
    S: SubscriptionType + 'static,
    E: GrandLineErrImpl,
{
    let v = value!({
        "id": id,
    });
    exec_assert_err(s, q, Some(v), e).await
}

async fn exec<Q, M, S>(s: &GraphQLSchema<Q, M, S>, q: &str, v: Option<GraphQLValue>) -> Response
where
    Q: ObjectType + Default + 'static,
    M: ObjectType + Default + 'static,
    S: SubscriptionType + 'static,
{
    let mut req = Request::new(q);
    if let Some(v) = v {
        req = req.variables(Variables::from_value(v));
    }
    s.execute(req).await
}
