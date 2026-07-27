use crate::prelude::*;

/// Identifier for the model's sea_orm entity type, e.g. Todo -> TodoSql.
pub fn ty_sql<D>(model: D) -> SynRes<Ts2>
where
    D: Display,
{
    format!("{model}_Sql").to_pascal_case().ts2_or_err()
}
/// Identifier for the model's async-graphql-facing type, e.g. Todo -> TodoGql.
pub fn ty_gql<D>(model: D) -> SynRes<Ts2>
where
    D: Display,
{
    format!("{model}_Gql").to_pascal_case().ts2_or_err()
}
/// Identifier for the model's sea_orm Column enum, e.g. Todo -> TodoColumn.
pub fn ty_column<D>(model: D) -> SynRes<Ts2>
where
    D: Display,
{
    format!("{model}_Column").to_pascal_case().ts2_or_err()
}
/// Identifier for the model's sea_orm ActiveModel type, e.g. Todo -> TodoActiveModel.
pub fn ty_am<D>(model: D) -> SynRes<Ts2>
where
    D: Display,
{
    format!("{model}_ActiveModel").to_pascal_case().ts2_or_err()
}
/// Identifier for the model's generated Filter input type, e.g. Todo -> TodoFilter.
pub fn ty_filter<D>(model: D) -> SynRes<Ts2>
where
    D: Display,
{
    format!("{model}_Filter").to_pascal_case().ts2_or_err()
}
/// Identifier for the model's generated OrderBy enum, e.g. Todo -> TodoOrderBy.
pub fn ty_order_by<D>(model: D) -> SynRes<Ts2>
where
    D: Display,
{
    format!("{model}_OrderBy").to_pascal_case().ts2_or_err()
}
