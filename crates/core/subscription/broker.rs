use super::prelude::*;
use async_graphql::futures_util::stream::BoxStream;

/// Fan-out of row change events. Pick one with SubscriptionConfig, or implement this and
/// wrap it in SubscriptionBroker::Custom for a transport the framework does not ship.
#[async_trait]
pub trait SubscriptionBrokerImpl
where
    Self: Send + Sync,
{
    /// Deliver one event to every subscriber of its entity.
    async fn publish(&self, e: SubscriptionEvent) -> Res<()>;
    /// Stream of events for entity, live from the moment it is called.
    fn subscribe(&self, entity: &'static str) -> BoxStream<'static, SubscriptionEvent>;
}

/// Which transport carries subscription events, the only thing an app has to
/// choose between a single instance and a horizontally scaled one.
#[derive(Clone)]
pub enum SubscriptionBroker {
    /// Process local channel, reaches only the subscribers of this instance.
    /// Correct for a single process, silently partial once there are two.
    InMemory,
    /// Redis pub/sub, every instance pointed at the same server sees every event.
    #[cfg(feature = "subscription_redis")]
    Redis(SubscriptionRedis),
    /// Any transport of your own.
    Custom(Arc<dyn SubscriptionBrokerImpl>),
}

impl SubscriptionBroker {
    /// Resolves the choice into the impl behind it. Nothing connects here, the
    /// redis adapter opens its connection on first use.
    pub fn into_impl(self) -> Arc<dyn SubscriptionBrokerImpl> {
        match self {
            Self::InMemory => Arc::new(InMemoryBroker::default()),
            #[cfg(feature = "subscription_redis")]
            Self::Redis(c) => Arc::new(RedisBroker::new(c)),
            Self::Custom(b) => b,
        }
    }
}
