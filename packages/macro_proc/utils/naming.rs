use crate::prelude::*;

pub fn ty_sql<D>(model: D) -> SynRes<Ts2>
where
    D: Display,
{
    format!("{model}_Sql").to_pascal_case().ts2_or_err()
}
pub fn ty_gql<D>(model: D) -> SynRes<Ts2>
where
    D: Display,
{
    format!("{model}_Gql").to_pascal_case().ts2_or_err()
}
pub fn ty_column<D>(model: D) -> SynRes<Ts2>
where
    D: Display,
{
    format!("{model}_Column").to_pascal_case().ts2_or_err()
}
pub fn ty_am<D>(model: D) -> SynRes<Ts2>
where
    D: Display,
{
    format!("{model}_ActiveModel").to_pascal_case().ts2_or_err()
}
pub fn ty_filter<D>(model: D) -> SynRes<Ts2>
where
    D: Display,
{
    format!("{model}_Filter").to_pascal_case().ts2_or_err()
}
pub fn ty_order_by<D>(model: D) -> SynRes<Ts2>
where
    D: Display,
{
    format!("{model}_OrderBy").to_pascal_case().ts2_or_err()
}
