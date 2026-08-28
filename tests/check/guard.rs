#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// check = guard, the bare form with no argument
// ---------------------------------------------------------------------------

#[tokio::test]
async fn bare_guard_allows_when_it_passes() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(Clearance::Agent).finish();

    let q = "
    query {
        patternPing
    }
    ";
    let expected = value!({
        "patternPing": true,
    });

    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn bare_guard_rejects_when_it_fails() -> Res<()> {
    let d = setup().await?;
    let s = d.s.finish();

    let q = "
    query {
        patternPing
    }
    ";

    exec_assert_err(&s, q, None, &FringeErr::ClearanceMissing).await?;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// check = guard(arg), the call form forwarding arguments
// ---------------------------------------------------------------------------

#[tokio::test]
async fn guard_with_arg_allows_matching_clearance() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(Clearance::Observer).finish();

    let q = "
    query {
        observerPing
    }
    ";
    let expected = value!({
        "observerPing": true,
    });

    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn guard_with_arg_rejects_other_clearance() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(Clearance::Agent).finish();

    let q = "
    query {
        observerPing
    }
    ";

    exec_assert_err(&s, q, None, &FringeErr::ClearanceDenied).await?;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// check(a, b), many guards running in declaration order
// ---------------------------------------------------------------------------

#[tokio::test]
async fn many_guards_allow_when_all_pass() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(Clearance::Observer).finish();

    let q = "
    query {
        septemberPing
    }
    ";
    let expected = value!({
        "septemberPing": true,
    });

    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn many_guards_stop_at_the_first_failing_one() -> Res<()> {
    let d = setup().await?;
    let s = d.s.finish();

    let q = "
    query {
        septemberPing
    }
    ";

    // Both guards would fail here, ClearanceMissing proves the first one ran first.
    exec_assert_err(&s, q, None, &FringeErr::ClearanceMissing).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn many_guards_reject_at_the_later_one() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(Clearance::Agent).finish();

    let q = "
    query {
        septemberPing
    }
    ";

    // The first guard passes with any clearance, so only the second can deny.
    exec_assert_err(&s, q, None, &FringeErr::ClearanceDenied).await?;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// check on a crud macro
// ---------------------------------------------------------------------------

#[tokio::test]
async fn crud_guard_allows_when_it_passes() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(Clearance::Agent).finish();

    am_create!(CaseFile {
        name: "The Pattern".to_owned(),
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    let q = "
    query {
        caseFileSearch {
            name
        }
    }
    ";
    let expected = value!({
        "caseFileSearch": [
            {
                "name": "The Pattern",
            },
        ],
    });

    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn crud_guard_works_on_a_mutation_without_the_auth_feature() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(Clearance::Agent).finish();

    let q = r#"
    mutation {
        caseFileCreate(data: { name: "Jacksonville" }) {
            name
        }
    }
    "#;
    let expected = value!({
        "caseFileCreate": {
            "name": "Jacksonville",
        },
    });

    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn crud_guard_rejects_before_the_query_runs() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(Clearance::Observer).finish();

    let q = "
    query {
        caseFileSearch {
            name
        }
    }
    ";

    exec_assert_err(&s, q, None, &FringeErr::ClearanceDenied).await?;

    d.tmp.drop().await
}
