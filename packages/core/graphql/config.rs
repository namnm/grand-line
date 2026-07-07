/// Configuration for pagination limits shared across the core graphql layer.
#[derive(Clone)]
pub struct CoreConfig {
    /// Limit applied to a query when no explicit limit is requested.
    pub limit_default: u64,
    /// Upper bound a requested limit is clamped to.
    pub limit_max: u64,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            limit_default: 10,
            limit_max: 100,
        }
    }
}
