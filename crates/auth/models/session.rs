use crate::prelude::*;

/// Marker trait for the consumer's login session model, implemented by the
/// #[auth_session] macro, backing a default AuthSessionImpl so the app does
/// not have to hand write the lookup.
pub trait AuthSessionModel
where
    Self: EntityX + Send + Sync,
{
    /// Converts a row into the minimal session the auth engine needs.
    fn auth_impl_session(m: Self::M) -> AuthImplSession;

    /// Build the default AuthSessionImpl for this entity, used unless a custom
    /// implementation is registered.
    fn auth_default_impl() -> Box<dyn AuthSessionImpl> {
        Box::new(DefaultSessionImpl::<Self>(PhantomData))
    }
}

/// Default AuthSessionImpl backed by any model type S implementing AuthSessionModel.
pub struct DefaultSessionImpl<S>(pub(crate) PhantomData<S>);

#[async_trait]
impl<S> AuthSessionImpl for DefaultSessionImpl<S>
where
    S: AuthSessionModel,
{
    async fn find(&self, ctx: &Context<'_>, id: &str) -> Res<Option<AuthImplSession>> {
        let db = &ctx.db().await?;
        let m = S::find().include_deleted(false).filter_by_id(id).one(db).await?;
        Ok(m.map(S::auth_impl_session))
    }
}
