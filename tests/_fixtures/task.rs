use super::check::*;
use grand_line::prelude::*;

#[model]
pub struct Task {
    pub title: String,
    pub assignee_id: String,
    pub org_id: String,
}

#[query(check = authz_org)]
fn tasks(order_by: Option<Vec<TaskOrderBy>>) -> Vec<TaskGql> {
    let filter = ctx.authz_row::<TaskFilter>().await?;
    Task::find()
        .filter_option(filter)
        .chain(order_by)
        .gql_select(ctx)?
        .all(db)
        .await?
}
