use super::prelude::*;

/// What happened to a row, carried on every subscription event.
#[gql_enum]
pub enum SubscriptionOperation {
    Create,
    Update,
    Delete,
}

/// One row change, queued by a mutation and published to the broker only after
/// the request transaction has committed.
#[derive(Clone, Debug)]
pub struct SubscriptionEvent {
    /// Model name of the entity the row belongs to, see EntityX::model_name.
    pub entity: &'static str,
    pub operation: SubscriptionOperation,
    pub id: String,
}
