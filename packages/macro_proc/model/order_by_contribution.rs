use crate::prelude::*;

/// Shared contract for anything that can add variants + query arms to a model's
/// OrderBy enum. Only scalar fields implement this today (Asc/Desc per column),
/// relations don't contribute order-by variants.
pub trait OrderByContribution {
    fn push_order_by(&self, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>) -> SynRes<()>;
}

impl OrderByContribution for Field {
    fn push_order_by(&self, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>) -> SynRes<()> {
        order_by(self, struk, query)
    }
}
