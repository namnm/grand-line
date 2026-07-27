use crate::prelude::*;

/// Shared contract for anything that can add fields + conditions to a model's Filter
/// struct: scalar fields (eq/ne/...) and relations (_some/_none/_every).
pub trait FilterContribution {
    fn push_filter(&self, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>) -> SynRes<()>;
}

impl FilterContribution for Field {
    fn push_filter(&self, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>) -> SynRes<()> {
        filter(self, struk, query)
    }
}

impl FilterContribution for GenRelation {
    fn push_filter(&self, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>) -> SynRes<()> {
        relation_filter(self, struk, query)
    }
}
