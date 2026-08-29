#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

pub use core::time::Duration;
pub use grand_line::prelude::*;

#[model]
pub struct Experiment {
    pub name: String,
    #[default(false)]
    pub classified: bool,
}

#[gql_input]
pub struct ExperimentCreate {
    pub name: String,
}

#[create(Experiment)]
fn resolver() {
    am_create!(Experiment {
        name: data.name,
    })
}

#[gql_input]
pub struct ExperimentUpdate {
    pub name: String,
}

#[update(Experiment)]
fn resolver() {
    am_update!(Experiment {
        id: id.clone(),
        name: data.name,
    })
}

#[delete(Experiment)]
fn resolver() {
}

/// A row written by a subscription resolver's detached job, nothing else
/// produces this name.
pub const DETACHED_NAME: &str = "detached-job";

/// Detaches a job writing a row, then lets the generated code finish the
/// transaction. cleanup never runs for a subscription, execute() is skipped
/// and the stream's lifetime is handled by TxRelease, so this job only runs if
/// tx_finish spawns the queue after its commit.
#[subscribe(Experiment)]
fn experiment_detached_changed() {
    ctx.detach(move |db| async move {
        am_create!(Experiment {
            name: DETACHED_NAME.to_owned(),
        })
        .exec_without_ctx(db.as_ref())
        .await?;
        Ok(())
    })
    .await?;
}

#[gql_input]
pub struct ExperimentCreateQuiet {
    pub name: String,
}

/// Publishes nothing, so a subscriber never hears about its rows.
#[create(Experiment, publish = false)]
fn experiment_create_quiet() {
    am_create!(Experiment {
        name: data.name,
    })
}

#[subscribe(Experiment)]
fn resolver() {
}

/// Server side filter, only the rows this subscription is scoped to reach the client.
#[subscribe(Experiment)]
fn experiment_open_changed() {
    filter!(Experiment {
        classified: false,
    })
    .into()
}

/// Opts in to events for rows that no longer exist, node comes back null.
#[subscribe(Experiment, allow_permanent_delete)]
fn experiment_gone_changed() {
}

#[derive(Default, MergedObject)]
pub struct Mutation(
    ExperimentCreateMutation,
    ExperimentUpdateMutation,
    ExperimentDeleteMutation,
    ExperimentCreateQuietMutation,
);
#[derive(Default, MergedSubscription)]
pub struct Subscription(
    ExperimentChangedSubscription,
    ExperimentOpenChangedSubscription,
    ExperimentGoneChangedSubscription,
    ExperimentDetachedChangedSubscription,
);

// ---------------------------------------------------------------------------
// Gql documents
// ---------------------------------------------------------------------------

pub const S_CHANGED: &str = "
subscription {
    experimentChanged {
        created {
            name
            classified
        }
        updated {
            name
            classified
        }
        deleted {
            id
            name
        }
    }
}
";

/// Only creates are selected, so nothing else should ever arrive.
pub const S_CREATED_ONLY: &str = "
subscription {
    experimentChanged {
        created {
            name
        }
    }
}
";

pub const S_OPEN_CHANGED: &str = "
subscription {
    experimentOpenChanged {
        created {
            name
        }
        updated {
            name
        }
    }
}
";

pub const S_GONE_CHANGED: &str = "
subscription {
    experimentGoneChanged {
        deleted {
            id
        }
    }
}
";

pub const S_DETACHED: &str = "
subscription {
    experimentDetachedChanged {
        created {
            name
        }
    }
}
";

pub const M_CREATE: &str = "
mutation test($data: ExperimentCreate!) {
    experimentCreate(data: $data) {
        id
    }
}
";

pub const M_CREATE_QUIET: &str = "
mutation test($data: ExperimentCreateQuiet!) {
    experimentCreateQuiet(data: $data) {
        id
    }
}
";

pub const M_UPDATE: &str = "
mutation test($id: String!, $data: ExperimentUpdate!) {
    experimentUpdate(id: $id, data: $data) {
        id
    }
}
";

pub const M_DELETE: &str = "
mutation test($id: String!, $permanent: Boolean) {
    experimentDelete(id: $id, permanent: $permanent) {
        id
    }
}
";

/// Variables creating an experiment named name.
pub fn v_create(name: &str) -> GraphQLValue {
    value!({
        "data": {
            "name": name,
        },
    })
}

/// Variables renaming the experiment id to name.
pub fn v_update(id: &str, name: &str) -> GraphQLValue {
    value!({
        "id": id,
        "data": {
            "name": name,
        },
    })
}

/// Variables deleting the experiment id, permanently when permanent.
pub fn v_delete(id: &str, permanent: bool) -> GraphQLValue {
    value!({
        "id": id,
        "permanent": permanent,
    })
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub struct Setup {
    pub tmp: TmpDb,
    pub s: GraphQLSchema<EmptyQuery, Mutation, Subscription>,
    /// The very config the schema was built with, the only one that reaches its
    /// subscribers, see InMemoryBroker.
    pub subscription: SubscriptionConfig,
}

pub async fn setup() -> Res<Setup> {
    let tmp = tmp_db!(Experiment);
    let subscription = SubscriptionConfig::default();
    let s = GraphQLSchema::build(EmptyQuery::default(), Mutation::default(), Subscription::default())
        .extension(GrandLineExtension)
        .data(Arc::new(tmp.db.clone()))
        .data(subscription.clone())
        .finish();

    Ok(Setup {
        tmp,
        s,
        subscription,
    })
}

/// Waits until the subscription is actually listening. The stream is lazy: the
/// resolver, and with it the broker subscribe, only runs on the first poll, and
/// the broker has no replay, so anything published before that first poll is gone.
/// A real client has the same race and handles it by fetching initial state after
/// subscribing.
pub async fn subscription_ready<S>(stream: &mut S)
where
    S: Stream<Item = Response> + Unpin,
{
    drop(timeout(Duration::from_secs(1), stream.next()).await);
}

/// Polls for the row a subscription resolver's detached job writes.
pub async fn wait_detached(tmp: &TmpDb) -> Res<u64> {
    let mut n = 0;
    for _ in 0..100u8 {
        n = Experiment::find()
            .filter(ExperimentColumn::Name.eq(DETACHED_NAME))
            .count(&tmp.db)
            .await?;
        if n >= 1 {
            return Ok(n);
        }
        sleep(Duration::from_millis(10)).await;
    }
    Ok(n)
}

/// Reads the next subscription payload, or fails the test after timing out so a
/// missing event never hangs the suite.
pub async fn next_event<S>(stream: &mut S) -> Res<JsonValue>
where
    S: Stream<Item = Response> + Unpin,
{
    let Ok(Some(r)) = timeout(Duration::from_secs(3), stream.next()).await else {
        return TestErr::expect("subscription should receive an event");
    };
    if let Some(e) = r.errors.first() {
        let m = &e.message;
        return TestErr::expect(&format!("subscription event should not error: {m}"));
    }
    r.data.to_json()
}
