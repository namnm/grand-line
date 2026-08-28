#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

pub use grand_line::prelude::*;

#[grand_line_err]
pub enum FringeErr {
    #[error("clearance is missing")]
    #[client]
    ClearanceMissing,
    #[error("clearance is denied")]
    #[client]
    ClearanceDenied,
}

// ---------------------------------------------------------------------------
// Guard trait, the only place the check logic lives, the macro just calls it
// ---------------------------------------------------------------------------

/// Stands in for whatever a real app resolves per request, registered on the
/// schema so the guards below need no http feature to read it.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Clearance {
    Agent,
    Observer,
}

#[async_trait]
pub trait FringeCheck {
    async fn cleared(&self) -> Res<()>;
    async fn clearance(&self, required: Clearance) -> Res<()>;
}

#[async_trait]
impl FringeCheck for Context<'_> {
    async fn cleared(&self) -> Res<()> {
        if self.data_opt::<Clearance>().is_none() {
            return Err(FringeErr::ClearanceMissing.into());
        }
        Ok(())
    }

    async fn clearance(&self, required: Clearance) -> Res<()> {
        self.cleared().await?;
        if self.data_opt::<Clearance>() != Some(&required) {
            return Err(FringeErr::ClearanceDenied.into());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Model and resolvers covering every check form
// ---------------------------------------------------------------------------

#[model]
pub struct CaseFile {
    pub name: String,
}

#[query(check = cleared)]
fn pattern_ping() -> bool {
    true
}

#[query(check = clearance(Clearance::Observer))]
fn observer_ping() -> bool {
    true
}

#[query(check(cleared, clearance(Clearance::Observer)))]
fn september_ping() -> bool {
    true
}

#[search(CaseFile, check = clearance(Clearance::Agent))]
fn resolver() {
}

#[gql_input]
pub struct CaseFileCreate {
    pub name: String,
}

#[create(CaseFile, check = clearance(Clearance::Agent))]
fn case_file_create() {
    am_create!(CaseFile {
        name: data.name,
    })
}

#[derive(Default, MergedObject)]
pub struct Query(
    PatternPingQuery,
    ObserverPingQuery,
    SeptemberPingQuery,
    CaseFileSearchQuery,
);
#[derive(Default, MergedObject)]
pub struct Mutation(CaseFileCreateMutation);

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub struct Setup {
    pub tmp: TmpDb,
    pub s: SchemaBuilder<Query, Mutation, EmptySubscription>,
}

pub async fn setup() -> Res<Setup> {
    let tmp = tmp_db!(CaseFile);
    let s = schema_qm::<Query, Mutation>(&tmp.db);

    Ok(Setup {
        tmp,
        s,
    })
}
