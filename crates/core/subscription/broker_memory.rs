use super::prelude::*;
use async_graphql::futures_util::stream::{self, BoxStream, StreamExt as _};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

/// How many events the channel buffers before the slowest subscriber starts
/// missing them.
const CAPACITY: usize = 1024;

/// Process local SubscriptionBrokerImpl, backed by one broadcast channel.
///
/// The channel belongs to this broker, not to the process, so two schemas built
/// with two configs never hear each other's events. Publishing from outside a
/// request therefore has to go through the same SubscriptionConfig the schema was
/// built with, cloned rather than rebuilt.
pub struct InMemoryBroker(broadcast::Sender<SubscriptionEvent>);

impl Default for InMemoryBroker {
    fn default() -> Self {
        Self(broadcast::channel(CAPACITY).0)
    }
}

#[async_trait]
impl SubscriptionBrokerImpl for InMemoryBroker {
    async fn publish(&self, e: SubscriptionEvent) -> Res<()> {
        // An error here only means nobody is subscribed right now, not a failure.
        drop(self.0.send(e));
        Ok(())
    }

    fn subscribe(&self, entity: &'static str) -> BoxStream<'static, SubscriptionEvent> {
        let rx = self.0.subscribe();
        stream::unfold(rx, |mut rx| async move {
            loop {
                match rx.recv().await {
                    Ok(e) => return Some((e, rx)),
                    Err(RecvError::Closed) => return None,
                    // Lagged means this subscriber fell behind and the channel
                    // dropped events for it, keep going with what is still buffered.
                    Err(RecvError::Lagged(_)) => (),
                }
            }
        })
        .filter(move |e| {
            let keep = e.entity == entity;
            async move { keep }
        })
        .boxed()
    }
}
