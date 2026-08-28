#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// Crud mutations publish, and only after they commit
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_reaches_the_subscriber_with_the_new_row() -> Res<()> {
    let d = setup().await?;
    let mut sub = d.s.execute_stream(S_CHANGED);
    subscription_ready(&mut sub).await;

    exec_assert_ok(&d.s, M_CREATE, Some(v_create("Cortexiphan"))).await;

    let r = next_event(&mut sub).await?;
    pretty_eq!(
        r.str("/experimentChanged/created/name"),
        "Cortexiphan",
        "created should carry the row as it stands now",
    );
    pretty_eq!(
        r.ptr("/experimentChanged/updated"),
        &json!(null),
        "only the slot matching the change should be set",
    );

    d.tmp.drop().await
}

#[tokio::test]
async fn update_and_delete_reach_the_subscriber_in_order() -> Res<()> {
    let d = setup().await?;
    let mut sub = d.s.execute_stream(S_CHANGED);
    subscription_ready(&mut sub).await;

    let r = exec_assert_ok(&d.s, M_CREATE, Some(v_create("Cortexiphan"))).await;
    let id = r.data.to_json()?.str("/experimentCreate/id").to_owned();

    // Each event is read before the next write, the payload carries the row as it
    // stands when the event is delivered, not a snapshot of when it was published.
    let r = next_event(&mut sub).await?;
    pretty_eq!(
        r.str("/experimentChanged/created/name"),
        "Cortexiphan",
        "first event is the create",
    );

    exec_assert_ok(&d.s, M_UPDATE, Some(v_update(&id, "Cortexiphan 2"))).await;
    let r = next_event(&mut sub).await?;
    pretty_eq!(
        r.str("/experimentChanged/updated/name"),
        "Cortexiphan 2",
        "second event is the update, carrying the new name",
    );

    exec_assert_ok(&d.s, M_DELETE, Some(v_delete(&id, false))).await;
    let r = next_event(&mut sub).await?;
    pretty_eq!(
        r.str("/experimentChanged/deleted/name"),
        "Cortexiphan 2",
        "a soft deleted row is still loaded so the client sees what disappeared",
    );

    d.tmp.drop().await
}

#[tokio::test]
async fn a_failed_mutation_publishes_nothing() -> Res<()> {
    let d = setup().await?;
    let mut sub = d.s.execute_stream(S_CHANGED);
    subscription_ready(&mut sub).await;

    // Unknown id, the update errors and the request rolls back.
    let r = exec(&d.s, M_UPDATE, Some(v_update("missing", "Ignored"))).await;
    pretty_eq!(r.errors.is_empty(), false, "updating a missing row should error");

    // A later successful mutation is the first thing the subscriber sees.
    exec_assert_ok(&d.s, M_CREATE, Some(v_create("Cortexiphan"))).await;

    let r = next_event(&mut sub).await?;
    pretty_eq!(
        r.str("/experimentChanged/created/name"),
        "Cortexiphan",
        "the rolled back update should not have been published",
    );

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// Opting out of publishing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn publish_false_keeps_the_row_off_the_stream() -> Res<()> {
    let d = setup().await?;
    let mut sub = d.s.execute_stream(S_CHANGED);
    subscription_ready(&mut sub).await;

    exec_assert_ok(&d.s, M_CREATE_QUIET, Some(v_create("ZFT"))).await;
    exec_assert_ok(&d.s, M_CREATE, Some(v_create("Cortexiphan"))).await;

    let r = next_event(&mut sub).await?;
    pretty_eq!(
        r.str("/experimentChanged/created/name"),
        "Cortexiphan",
        "the publish = false mutation should be skipped entirely",
    );

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// The subscription body filters events server side
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_row_outside_the_body_filter_is_skipped() -> Res<()> {
    let d = setup().await?;
    let mut sub = d.s.execute_stream(S_OPEN_CHANGED);
    subscription_ready(&mut sub).await;

    let classified = am_create!(Experiment {
        name: "The Pattern",
        classified: true,
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    // Touching the classified row publishes an event the filter drops.
    exec_assert_ok(&d.s, M_UPDATE, Some(v_update(&classified.id, "Redacted"))).await;
    exec_assert_ok(&d.s, M_CREATE, Some(v_create("Cortexiphan"))).await;

    let r = next_event(&mut sub).await?;
    pretty_eq!(
        r.str("/experimentOpenChanged/created/name"),
        "Cortexiphan",
        "only the row matching the subscription filter should arrive",
    );

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// Selecting only some operations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn an_operation_the_client_did_not_select_never_arrives() -> Res<()> {
    let d = setup().await?;
    let mut sub = d.s.execute_stream(S_CREATED_ONLY);
    subscription_ready(&mut sub).await;

    let r = exec_assert_ok(&d.s, M_CREATE, Some(v_create("Cortexiphan"))).await;
    let id = r.data.to_json()?.str("/experimentCreate/id").to_owned();

    let r = next_event(&mut sub).await?;
    pretty_eq!(r.str("/experimentChanged/created/name"), "Cortexiphan", "first create");

    // Only creates were selected, so this update must not produce an event.
    exec_assert_ok(&d.s, M_UPDATE, Some(v_update(&id, "Ignored"))).await;
    exec_assert_ok(&d.s, M_CREATE, Some(v_create("Second"))).await;

    let r = next_event(&mut sub).await?;
    pretty_eq!(
        r.str("/experimentChanged/created/name"),
        "Second",
        "the update in between should have been dropped before any query ran",
    );

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// Rows that no longer exist
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_permanently_deleted_row_is_skipped_by_default() -> Res<()> {
    let d = setup().await?;
    let mut sub = d.s.execute_stream(S_CHANGED);
    subscription_ready(&mut sub).await;

    let gone = am_create!(Experiment {
        name: "Vanished",
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    exec_assert_ok(&d.s, M_DELETE, Some(v_delete(&gone.id, true))).await;
    exec_assert_ok(&d.s, M_CREATE, Some(v_create("Cortexiphan"))).await;

    let r = next_event(&mut sub).await?;
    pretty_eq!(
        r.str("/experimentChanged/created/name"),
        "Cortexiphan",
        "a row that cannot be reloaded should not reach a subscription without the opt in",
    );

    d.tmp.drop().await
}

#[tokio::test]
async fn allow_permanent_delete_still_resolves_the_id() -> Res<()> {
    let d = setup().await?;
    let mut sub = d.s.execute_stream(S_GONE_CHANGED);
    subscription_ready(&mut sub).await;

    let gone = am_create!(Experiment {
        name: "Vanished",
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    exec_assert_ok(&d.s, M_DELETE, Some(v_delete(&gone.id, true))).await;

    let r = next_event(&mut sub).await?;
    pretty_eq!(
        r.str("/experimentGoneChanged/deleted/id"),
        gone.id,
        "the id of the removed row should still be selectable once it is gone",
    );

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// Publishing from outside a request
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_background_job_can_publish_without_a_request() -> Res<()> {
    let d = setup().await?;
    let mut sub = d.s.execute_stream(S_CHANGED);
    subscription_ready(&mut sub).await;

    let e = am_create!(Experiment {
        name: "Walter",
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    // No resolver, no ctx, going through the very config the schema was built with.
    d.subscription
        .publish::<Experiment>(SubscriptionOperation::Update, &e.id)
        .await?;

    let r = next_event(&mut sub).await?;
    pretty_eq!(
        r.str("/experimentChanged/updated/name"),
        "Walter",
        "an event published outside a request should reach the subscriber",
    );

    d.tmp.drop().await
}
