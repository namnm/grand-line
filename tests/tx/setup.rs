#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

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

#[derive(Default, MergedObject)]
pub struct Query(
    InvestigationDetailQuery,
    InvestigationConnIsTxQuery,
    InvestigationForcedTxIsTxQuery,
);
#[derive(Default, MergedObject)]
pub struct Mutation(InvestigationCreateMutation);

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub struct Setup {
    pub tmp: TmpDb,
    pub s: GraphQLSchema<Query, Mutation, EmptySubscription>,
}

pub async fn setup() -> Res<Setup> {
    let tmp = tmp_db!(Investigation, Report);
    let s = schema_qm::<Query, Mutation>(&tmp.db).finish();

    Ok(Setup {
        tmp,
        s,
    })
}
