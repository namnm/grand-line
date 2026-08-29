#[path = "./setup.rs"]
mod setup;
use setup::*;

/// Executes q with an operationName, the way a real client picks one operation
/// out of a multi-operation document.
async fn exec_named<Q, M, S>(s: &GraphQLSchema<Q, M, S>, q: &str, op: &str) -> Response
where
    Q: ObjectType + Default + 'static,
    M: ObjectType + Default + 'static,
    S: SubscriptionType + 'static,
{
    let req = Request::new(q).operation_name(op);
    s.execute(req).await
}

// ---------------------------------------------------------------------------
// Only the selected operation decides the transaction
// ---------------------------------------------------------------------------

/// A document carrying both a query and a mutation. Selecting the query with
/// operationName must classify the request as a read: the unused mutation
/// previously dragged the whole request onto the transaction path, opening and
/// pinning a connection the selected operation never needed.
const BOTH: &str = "
query Cheap {
    investigationConnIsTx
}

mutation Unused {
    detachedReport(title: \"nope\")
}
";

#[tokio::test]
async fn the_selected_query_is_not_put_on_the_tx_path_by_an_unused_mutation() -> Res<()> {
    let d = setup().await?;

    let r = exec_named(&d.s, BOTH, "Cheap").await;
    pretty_eq!(r.errors.is_empty(), true, "the selected query should run cleanly",);
    let data = r.data.to_json()?;
    pretty_eq!(
        data.ptr("/investigationConnIsTx"),
        &json!(false),
        "a named query next to an unused mutation must not open a transaction",
    );

    d.tmp.drop().await
}

#[tokio::test]
async fn the_selected_mutation_still_runs_on_the_tx_path() -> Res<()> {
    let d = setup().await?;

    let r = exec_named(&d.s, BOTH, "Unused").await;
    pretty_eq!(r.errors.is_empty(), true, "the selected mutation should run cleanly",);
    let data = r.data.to_json()?;
    pretty_eq!(
        data.ptr("/detachedReport"),
        &json!(true),
        "the named mutation should have run",
    );
    // and its detached job still runs once the commit landed
    pretty_eq!(
        wait_reports(&d.tmp, 1).await?,
        1,
        "the selected mutation's detached job should run after the commit",
    );

    d.tmp.drop().await
}

// Control: a named query alone behaves like an anonymous one, no transaction.
#[tokio::test]
async fn a_named_query_alone_still_avoids_the_tx() -> Res<()> {
    let d = setup().await?;

    let r = exec_named(&d.s, "query Cheap { investigationConnIsTx }", "Cheap").await;
    pretty_eq!(r.errors.is_empty(), true, "the query should run cleanly");
    let data = r.data.to_json()?;
    pretty_eq!(
        data.ptr("/investigationConnIsTx"),
        &json!(false),
        "a lone query should not open a transaction",
    );

    d.tmp.drop().await
}
