use super::prelude::*;

/// Helper trait to combine `order_by` and `order_by_default` with an initial value if all are empty.
pub trait OrderBy<E>
where
    E: EntityX,
    Self: ChainSelect<E> + Serialize + Send + Sync,
{
    /// Get `order_by_default` to use in abstract methods.
    /// Should be generated in the model macro.
    fn conf_default() -> Self;
}

/// Automatically implement combine for Option<Vec<OrderBy>>.
pub trait OrderByImpl<E, O>
where
    E: EntityX,
    O: OrderBy<E>,
{
    /// Helper to combine `order_by` and `order_by_default` with an initial value if all are empty.
    fn combine(self, order_by_default: Self) -> Vec<O>;
}

/// Automatically implement combine for Option<Vec<OrderBy>>.
impl<E, O> OrderByImpl<E, O> for Option<Vec<O>>
where
    E: EntityX,
    O: OrderBy<E>,
{
    fn combine(self, order_by_default: Self) -> Vec<O> {
        match self {
            Some(o) => {
                if o.is_empty() {
                    combine_option(order_by_default, O::conf_default())
                } else {
                    o
                }
            }
            None => combine_option(order_by_default, O::conf_default()),
        }
    }
}

fn combine_option<O>(o: Option<Vec<O>>, order_by_default: O) -> Vec<O> {
    match o {
        Some(a) => combine_vec(a, order_by_default),
        None => vec![order_by_default],
    }
}
fn combine_vec<O>(o: Vec<O>, order_by_default: O) -> Vec<O> {
    if o.is_empty() {
        vec![order_by_default]
    } else {
        o
    }
}
