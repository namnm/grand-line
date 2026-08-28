/// Configuration for pagination limits shared across the core graphql layer.
#[derive(Clone)]
pub struct CoreConfig {
    /// Limit applied to a query when no explicit limit is requested.
    pub limit_default: u64,
    /// Upper bound a requested limit is clamped to.
    pub limit_max: u64,
    /// Upper bound a requested offset is clamped to.
    /// A deep offset is a full scan the database cannot shortcut, from one query
    /// that otherwise looks ordinary.
    pub offset_max: u64,
    /// Upper bound on how many order_by entries a request may send.
    /// Only the client supplied list is capped, not the app own default.
    pub order_by_max: usize,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            limit_default: 10,
            limit_max: 100,
            offset_max: 10_000,
            order_by_max: 5,
        }
    }
}
