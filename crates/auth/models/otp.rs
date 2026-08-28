use crate::prelude::*;

/// Marker trait for the consumer's otp model, implemented by the #[auth_otp]
/// macro, backing a default AuthOtpImpl so the app does not have to hand write
/// the four lookup and mutation queries.
pub trait AuthOtpModel
where
    Self: EntityX + Send + Sync,
{
    /// Get column ty.
    fn col_ty() -> Self::C;
    /// Get column email.
    fn col_email() -> Self::C;
    /// Get column total_attempt.
    fn col_total_attempt() -> Self::C;

    /// Converts a row into the otp the auth engine verifies against.
    fn auth_impl_otp(m: Self::M) -> AuthImplOtp;

    /// Build the default AuthOtpImpl for this entity, used unless a custom
    /// implementation is registered.
    fn auth_default_impl() -> Box<dyn AuthOtpImpl> {
        Box::new(DefaultOtpImpl::<Self>(PhantomData))
    }
}

/// Default AuthOtpImpl backed by any model type O implementing AuthOtpModel.
/// Runs on the plain db connection, not the request transaction, so an attempt
/// is persisted even when the overall request later errors and rolls back.
pub struct DefaultOtpImpl<O>(pub(crate) PhantomData<O>);

#[async_trait]
impl<O> AuthOtpImpl for DefaultOtpImpl<O>
where
    O: AuthOtpModel,
{
    async fn find(&self, ctx: &Context<'_>, ty: &str, email: &str) -> Res<Option<AuthImplOtp>> {
        let db = ctx.db_pool().await?;
        let m = O::find()
            .include_deleted(false)
            .filter(O::col_ty().eq(ty))
            .filter(O::col_email().eq(email))
            .one(db)
            .await?;
        Ok(m.map(O::auth_impl_otp))
    }

    async fn increment(&self, ctx: &Context<'_>, id: &str, ty: &str) -> Res<Option<AuthImplOtp>> {
        let db = ctx.db_pool().await?;
        let u = O::update_many()
            .include_deleted(false)
            .filter_by_id(id)
            .filter(O::col_ty().eq(ty))
            .set(O::A::defaults_on_update())
            .col_expr(O::col_total_attempt(), Expr::col(O::col_total_attempt()).add(1));

        if u.exec(db).await?.rows_affected == 0 {
            return Ok(None);
        }
        let m = O::find().include_deleted(false).filter_by_id(id).one(db).await?;
        Ok(m.map(O::auth_impl_otp))
    }

    async fn reset(&self, ctx: &Context<'_>, id: &str) -> Res<()> {
        let db = ctx.db_pool().await?;
        O::update_many()
            .filter_by_id(id)
            .col_expr(O::col_total_attempt(), Expr::value(0))
            .exec(db)
            .await?;
        Ok(())
    }

    async fn delete(&self, ctx: &Context<'_>, ty: &str, email: &str) -> Res<()> {
        let db = ctx.db_pool().await?;
        O::delete_many()
            .filter(O::col_ty().eq(ty))
            .filter(O::col_email().eq(email))
            .exec(db)
            .await?;
        Ok(())
    }
}
