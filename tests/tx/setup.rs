#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

pub use core::time::Duration;
pub use grand_line::prelude::*;

// ---------------------------------------------------------------------------
// Models, the has_many drives a dataloader so a request selecting reports goes
// through LoaderX and its spawned batch task
// ---------------------------------------------------------------------------

#[model]
pub struct Investigation {
    pub name: String,
    #[has_many]
    pub reports: Report,
}

#[model]
pub struct Report {
    pub title: String,
    pub investigation_id: String,
}

#[gql_input]
pub struct InvestigationCreate {
    pub name: String,
}

#[create(Investigation)]
fn resolver() {
    am_create!(Investigation {
        name: data.name,
    })
}

#[detail(Investigation)]
fn resolver() {
}

// ---------------------------------------------------------------------------
// A read that only reads, to show it never opens a transaction
// ---------------------------------------------------------------------------

#[query]
fn investigation_conn_is_tx() -> bool {
    ctx.db().await?.is_tx()
}

/// Forces the transaction open even though the operation is a query, the escape
/// hatch for a read that turns out to write.
#[query]
fn investigation_forced_tx_is_tx() -> bool {
    ctx.tx().await?;
    ctx.db().await?.is_tx()
}

/// A read that errors, nullable so the error does not propagate to the root and
/// the response keeps whatever its sibling field produced.
#[query]
fn investigation_missing() -> Option<bool> {
    let db = &ctx.db().await?;
    Investigation::find_by_id(MISSING_ID).one_or_404(db).await?;
    Some(true)
}

// ---------------------------------------------------------------------------
// Detached jobs, queued during the request and spawned only after it commits
// ---------------------------------------------------------------------------

/// Queues a detached job writing a Report, so a test can see whether it ran.
#[mutation]
fn detached_report(title: String) -> bool {
    ctx.detach(move |db| async move {
        am_create!(Report {
            title,
            investigation_id: DETACHED_ID.to_owned(),
        })
        .exec_without_ctx(db.as_ref())
        .await?;
        Ok(())
    })
    .await?;
    true
}

/// Queues the same job, writes through the request transaction, then fails.
/// Nullable so the error stays on this field and data still carries the sibling
/// mutation's result, which is the case the rollback null out is about.
#[mutation]
fn detached_report_then_fail(title: String) -> Option<bool> {
    let db = &ctx.db().await?;
    ctx.detach(move |db| async move {
        am_create!(Report {
            title,
            investigation_id: DETACHED_ID.to_owned(),
        })
        .exec_without_ctx(db.as_ref())
        .await?;
        Ok(())
    })
    .await?;
    Investigation::find_by_id(MISSING_ID).one_or_404(db).await?;
    Some(true)
}

#[derive(Default, MergedObject)]
pub struct Query(
    InvestigationDetailQuery,
    InvestigationConnIsTxQuery,
    InvestigationForcedTxIsTxQuery,
    InvestigationMissingQuery,
);
#[derive(Default, MergedObject)]
pub struct Mutation(
    InvestigationCreateMutation,
    DetachedReportMutation,
    DetachedReportThenFailMutation,
);

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

/// An id no row ever has, so a lookup on it errors Db404.
pub const MISSING_ID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
/// Marks the rows a detached job writes, so a test can count only those.
pub const DETACHED_ID: &str = "detached";

pub struct Setup {
    pub tmp: TmpDb,
    pub s: GraphQLSchema<Query, Mutation, EmptySubscription>,
}

/// Polls for reports written by a detached job, which runs in a spawned task
/// after the response is already built.
pub async fn wait_reports(tmp: &TmpDb, want: u64) -> Res<u64> {
    let mut n = 0;
    for _ in 0..50u8 {
        n = Report::find()
            .filter(ReportColumn::InvestigationId.eq(DETACHED_ID))
            .count(&tmp.db)
            .await?;
        if n >= want {
            return Ok(n);
        }
        sleep(Duration::from_millis(10)).await;
    }
    Ok(n)
}

pub async fn setup() -> Res<Setup> {
    let tmp = tmp_db!(Investigation, Report);
    let s = schema_qm::<Query, Mutation>(&tmp.db).finish();

    Ok(Setup {
        tmp,
        s,
    })
}
