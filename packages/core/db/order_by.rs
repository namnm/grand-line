use super::prelude::*;

/// Helper trait to combine order_by and order_by_default with an initial value if all are empty.
pub trait OrderBy
where
    Self: ChainSelect<Self::E> + Clone + Copy + Serialize + Send + Sync,
{
    type E: EntityX;
    /// Get order_by_default to use in abstract methods.
    /// Should be generated in the model macro.
    fn conf_default() -> Self;
}

/// Automatically implement combine for Option<Vec<OrderBy>>.
pub trait OrderByImpl<O>
where
    O: OrderBy,
{
    /// Helper to combine order_by and order_by_default with an initial value if all are empty.
    fn combine(self, order_by_default: Vec<O>) -> Vec<O>;
}

/// Automatically implement combine for Option<Vec<OrderBy>>.
impl<O> OrderByImpl<O> for Option<Vec<O>>
where
    O: OrderBy,
{
    fn combine(self, order_by_default: Vec<O>) -> Vec<O> {
        match self {
            Some(o) if !o.is_empty() => o,
            _ => {
                if order_by_default.is_empty() {
                    vec![O::conf_default()]
                } else {
                    order_by_default
                }
            }
        }
    }
}
