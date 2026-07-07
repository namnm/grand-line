use super::prelude::*;

/// Abstract extra model methods implementation.
#[async_trait]
pub trait ModelX<E>
where
    E: EntityX<M = Self>,
    Self: FromQueryResult + ModelTrait + IntoActiveModel<E::A> + Serialize + Send + Sync,
{
    /// Get entity id as a string.
    /// Should be generated in the model macro.
    fn get_id(&self) -> String;
    /// Convert sql model to gql model, without checking virtual fields from context.
    /// Should be generated in the model macro.
    fn into_gql_without_look_ahead(self) -> E::G;

    /// Convert sql model to gql model, with checking virtual fields from context.
    async fn into_gql(self, ctx: &Context<'_>) -> Res<E::G> {
        let r = if E::gql_look_ahead(ctx)?.iter().any(|l| l.expr.is_some()) {
            let tx = &*ctx.tx().await?;
            let id = self.get_id();
            E::find()
                .filter_by_id(&id)
                .gql_select(ctx)?
                .one(tx)
                .await?
                .ok_or(MyErr::Db404)?
        } else {
            self.into_gql_without_look_ahead()
        };
        Ok(r)
    }
}
