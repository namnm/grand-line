#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// tx_finish spawns the detached queue, subscriptions have no other chance
// ---------------------------------------------------------------------------

// A subscription resolver queues a detached job and then finishes the
// transaction itself. cleanup never runs for a subscription, so before
// tx_finish spawned the queue the job was queued forever and detach's promise
// to run it after the request commits was silently broken.
#[tokio::test]
async fn a_subscription_resolver_detached_job_runs_after_tx_finish() -> Res<()> {
    let d = setup().await?;
    let mut sub = d.s.execute_stream(S_DETACHED);
    subscription_ready(&mut sub).await;

    pretty_eq!(
        wait_detached(&d.tmp).await?,
        1,
        "tx_finish should spawn the detached job after its commit",
    );

    d.tmp.drop().await
}
