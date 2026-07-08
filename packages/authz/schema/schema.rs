use crate::prelude::*;

/// GraphQL mutations for the authz package that need to be generic over the
/// host app's user entity U.
pub struct AuthzUserImplMutation<U>(PhantomData<U>)
where
    U: AuthUser;

impl<U> Default for AuthzUserImplMutation<U>
where
    U: AuthUser,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[Object]
impl<U> AuthzUserImplMutation<U>
where
    U: AuthUser,
{
    async fn org_invitation_resolve(&self, ctx: &Context<'_>, data: OtpResolve) -> Res<UserInRoleGql> {
        org_invitation_accept_impl::<U>(ctx, data).await
    }

    async fn system_impersonate(
        &self,
        ctx: &Context<'_>,
        user_id: String,
        reason: String,
    ) -> Res<LoginSessionWithSecret> {
        system_impersonate_impl::<U>(ctx, user_id, reason).await
    }
}

/// GraphQL queries for the authz package that need to be generic over the
/// host app's user entity U.
pub struct AuthzUserImplQuery<U>(PhantomData<U>)
where
    U: AuthUser;

impl<U> Default for AuthzUserImplQuery<U>
where
    U: AuthUser,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[Object]
impl<U> AuthzUserImplQuery<U>
where
    U: AuthUser,
{
    async fn my_org_invitations(&self, ctx: &Context<'_>) -> Res<Vec<OrgInvitationView>> {
        my_org_invitations_impl::<U>(ctx).await
    }
}

// ---------------------------------------------------------------------------
// Merged schema roots
// ---------------------------------------------------------------------------

/// Combined GraphQL query root for the authz package, merge into the host schema.
#[derive(Default, MergedObject)]
pub struct AuthzMergedQuery<U>(
    OrgRoleSearchQuery,
    OrgRoleCountQuery,
    OrgRoleDetailQuery,
    SystemRoleSearchQuery,
    SystemRoleCountQuery,
    SystemRoleDetailQuery,
    OrgUserInRoleSearchQuery,
    OrgUserInRoleCountQuery,
    OrgUserInRoleDetailQuery,
    SystemUserInRoleSearchQuery,
    SystemUserInRoleCountQuery,
    SystemUserInRoleDetailQuery,
    OrgImpersonationSearchQuery,
    OrgImpersonationCountQuery,
    OrgImpersonationDetailQuery,
    SystemImpersonationSearchQuery,
    SystemImpersonationCountQuery,
    SystemImpersonationDetailQuery,
    AuthzUserImplQuery<U>,
)
where
    U: AuthUser;

/// Combined GraphQL mutation root for the authz package, merge into the host schema.
#[derive(Default, MergedObject)]
pub struct AuthzMergedMutation<U>(
    OrgRoleCreateMutation,
    OrgRoleUpdateMutation,
    OrgRoleDeleteMutation,
    SystemRoleCreateMutation,
    SystemRoleUpdateMutation,
    SystemRoleDeleteMutation,
    OrgUserInRoleCreateMutation,
    OrgUserInRoleDeleteMutation,
    OrgInvitationCreateMutation,
    SystemInvitationCreateMutation,
    OrgInvitationRejectMutation,
    OrgImpersonateMutation,
    OrgImpersonateRevokeMutation,
    SystemImpersonateRevokeMutation,
    AuthzUserImplMutation<U>,
)
where
    U: AuthUser;
