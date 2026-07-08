pub use grand_line::prelude::*;

#[derive(Default)]
struct Query;
#[Object]
impl Query {
    async fn one(&self) -> i64 {
        0
    }
    async fn add(&self, a: i64, b: i64) -> i64 {
        a + b
    }
}

fn schema() -> GraphQLSchema<Query, EmptyMutation, EmptySubscription> {
    GraphQLSchema::build(Query, EmptyMutation, EmptySubscription).finish()
}

#[tokio::test]
async fn on_parse_error() {
    let s = schema();

    let r = s.execute("{ one( }").await;
    assert!(r.errors.len() == 1, "response should have an error");

    let Some(err) = &r.errors.first() else {
        return;
    };

    assert!(err.source.is_none(), "parse request error source should be none");
    assert!(err.path.is_empty(), "parse request error path should be empty");
}

#[tokio::test]
async fn on_unknown_field() {
    let s = schema();

    let r = s.execute("{ unknown }").await;
    assert!(r.errors.len() == 1, "response should have an error");

    let Some(err) = &r.errors.first() else {
        return;
    };

    assert!(err.source.is_none(), "unknown field error source should be none");
    assert!(err.path.is_empty(), "unknown field error path should be empty");
}

#[tokio::test]
async fn on_variable_type_mismatch() {
    let s = schema();

    let q = "
      query($a: Int!, $b: Int!) {
        add(a: $a, b: $b)
      }
    ";
    let v = value!({
        "a": 0,
        "b": "not integer"
    });

    let r = s.execute(Request::new(q).variables(Variables::from_value(v))).await;
    assert!(r.errors.len() == 1, "response should have an error");

    let Some(err) = &r.errors.first() else {
        return;
    };

    assert!(
        err.source.is_none(),
        "variable type mismatch error source should be none"
    );
    assert!(err.path.is_empty(), "variable type mismatch error path should be empty");
}
