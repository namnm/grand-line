use super::prelude::*;

/// Queues row changes for the subscription broker.
#[async_trait]
pub trait SubscriptionContext<'a>
where
    Self: GrandLineDataContext<'a>,
{
    /// Queue a row change to publish once the request transaction commits, so a
    /// rolled back request publishes nothing. The crud macros call this for you,
    /// a hand written mutation calls it itself.
    async fn subscription_queue<E>(&self, operation: SubscriptionOperation, id: &str) -> Res<()>
    where
        E: EntityX,
    {
        let e = SubscriptionEvent {
            entity: E::model_name(),
            operation,
            id: id.to_owned(),
        };
        self.grand_line()?.events.lock().await.push(e);
        Ok(())
    }
}

#[async_trait]
impl<'a> SubscriptionContext<'a> for Context<'a> {
}
