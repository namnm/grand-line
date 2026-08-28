use super::prelude::*;

/// Runtime configuration for subscriptions, register it on the schema to pick a
/// transport, omit it to get SubscriptionBroker::InMemory.
#[derive(Clone)]
pub struct SubscriptionConfig {
    broker: Arc<dyn SubscriptionBrokerImpl>,
}

impl SubscriptionConfig {
    /// Resolves broker into the impl behind it, done once at setup so a publish
    /// never pays for the choice.
    pub fn new(broker: SubscriptionBroker) -> Self {
        Self {
            broker: broker.into_impl(),
        }
    }

    /// The resolved broker backing this config.
    pub fn broker(&self) -> &Arc<dyn SubscriptionBrokerImpl> {
        &self.broker
    }

    /// Publishes a change straight away, for code with no request context such as
    /// a background job, a migration, or a separate worker process. A resolver
    /// should call ctx.subscription_queue instead, so its event waits for the
    /// request transaction to commit.
    pub async fn publish<E>(&self, operation: SubscriptionOperation, id: &str) -> Res<()>
    where
        E: EntityX,
    {
        let e = SubscriptionEvent {
            entity: E::model_name(),
            operation,
            id: id.to_owned(),
        };
        self.broker.publish(e).await
    }
}

impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self::new(SubscriptionBroker::InMemory)
    }
}
