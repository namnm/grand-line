use crate::prelude::*;

/// GraphQL mutations for register/login/forgot wired to the host app's user entity U.
pub struct AuthUserImplMutation<U>(PhantomData<U>)
where
    U: AuthUser;

impl<U> Default for AuthUserImplMutation<U>
where
    U: AuthUser,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[Object]
impl<U> AuthUserImplMutation<U>
where
    U: AuthUser,
{
    async fn register(&self, ctx: &Context<'_>, data: Register) -> Res<AuthOtpWithSecret> {
        register_impl::<U>(ctx, data).await
    }

    async fn register_resolve(&self, ctx: &Context<'_>, data: AuthOtpResolve) -> Res<LoginSessionWithSecret> {
        register_resolve_impl::<U>(ctx, data).await
    }

    async fn login(&self, ctx: &Context<'_>, data: Login) -> Res<LoginSessionWithSecret> {
        login_impl::<U>(ctx, data).await
    }

    async fn forgot(&self, ctx: &Context<'_>, data: Forgot) -> Res<AuthOtpWithSecret> {
        forgot_impl::<U>(ctx, data).await
    }

    async fn forgot_resolve(
        &self,
        ctx: &Context<'_>,
        data: AuthOtpResolve,
        password: String,
    ) -> Res<LoginSessionWithSecret> {
        forgot_resolve_impl::<U>(ctx, data, password).await
    }
}

// ---------------------------------------------------------------------------
// Merged schema roots
// ---------------------------------------------------------------------------

/// Combined GraphQL query root for the auth package, merge into the host schema.
#[derive(Default, MergedObject)]
pub struct AuthMergedQuery(
    LoginSessionCurrentQuery,
    LoginSessionSearchQuery,
    LoginSessionCountQuery,
);

/// Combined GraphQL mutation root for the auth package, merge into the host schema.
#[derive(Default, MergedObject)]
pub struct AuthMergedMutation<U>(
    LoginSessionDeleteMutation,
    LoginSessionDeleteAllMutation,
    LogoutMutation,
    AuthOtpResolveMutation,
    AuthUserImplMutation<U>,
)
where
    U: AuthUser;
