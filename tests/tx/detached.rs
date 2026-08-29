#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// A detached job runs only after the request commits
// ---------------------------------------------------------------------------

#[tokio::test]
async fn detached_job_runs_after_a_successful_commit() -> Res<()> {
    let d = setup().await?;

    let q = r#"
    mutation {
        detachedReport(title: "The Pattern")
    }
    "#;
    let expected = value!({
        "detachedReport": true,
    });
    exec_assert(&d.s, q, None, &expected).await;

    pretty_eq!(
        wait_reports(&d.tmp, 1).await?,
        1,
        "a detached job should run once the request committed",
    );

    d.tmp.drop().await
}

#[tokio::test]
async fn detached_job_is_dropped_when_the_request_rolls_back() -> Res<()> {
    let d = setup().await?;

    let q = r#"
    mutation {
        detachedReportThenFail(title: "The Pattern")
    }
    "#;
    let r = exec(&d.s, q, None).await;

    pretty_eq!(
        r.errors.is_empty(),
        false,
        "the failing mutation should report an error"
    );
    pretty_eq!(
        wait_reports(&d.tmp, 1).await?,
        0,
        "there is no background work to do for a request that did not land",
    );

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// data is nulled when a transaction was actually rolled back
// ---------------------------------------------------------------------------

#[tokio::test]
async fn rollback_nulls_the_response_data() -> Res<()> {
    let d = setup().await?;

    // the create runs first and succeeds, then the second field fails and takes the
    // whole transaction down with it, so what the create returned no longer exists
    let q = r#"
    mutation {
        created: investigationCreate(data: { name: "Northwest Passage" }) {
            id
        }
        failed: detachedReportThenFail(title: "The Pattern")
    }
    "#;
    let r = exec(&d.s, q, None).await;

    pretty_eq!(
        r.errors.is_empty(),
        false,
        "the failing mutation should report an error"
    );
    pretty_eq!(
        r.data,
        GraphQLValue::Null,
        "data produced by a rolled back mutation should not be returned",
    );
    pretty_eq!(
        Investigation::find().count(&d.tmp.db).await?,
        0,
        "the rolled back create should have written nothing",
    );

    d.tmp.drop().await
}

#[tokio::test]
async fn a_failing_read_keeps_partial_success() -> Res<()> {
    let d = setup().await?;

    // a query opens no transaction, so it undid nothing and graphql's partial
    // success still applies
    let q = "
    query {
        connIsTx: investigationConnIsTx
        missing: investigationMissing
    }
    ";
    let r = exec(&d.s, q, None).await;

    pretty_eq!(r.errors.is_empty(), false, "the failing read should report an error");
    pretty_eq!(
        r.data.to_json()?.ptr("/connIsTx").is_null(),
        false,
        "a read that rolled nothing back should keep the fields that succeeded",
    );

    d.tmp.drop().await
}
