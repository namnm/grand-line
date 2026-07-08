use crate::prelude::*;

/// Accepts a pending org invitation for the currently authenticated user, creating
/// a UserInRole for them. The otp's email must match the authenticated user's email.
pub async fn org_invitation_accept_impl<U>(ctx: &Context<'_>, data: OtpResolve) -> Res<UserInRoleGql>
where
    U: AuthUser,
{
    ctx.auth_ensure_authenticated().await?;

    let tx = &*ctx.tx().await?;
    let h = &ctx.authz_config().handlers;

    let t = otp_ensure_resolve(ctx, tx, OTP_TY_ORG_INVITATION, data).await?;

    let user_id = ctx.auth().await?;
    let u = U::find().filter(U::col_id().eq(&user_id)).one_or_404(tx).await?;
    if !U::get_email(&u).eq_ignore_ascii_case(&t.email) {
        return Err(MyErr::InvitationEmailMismatch.into());
    }

    let d = OtpDataOrgInvitation::from_json(t.data)?;

    let uir = am_create!(UserInRole {
        user_id: user_id.clone(),
        role_id: d.role_id,
        org_id: Some(d.org_id),
    })
    .exec_without_ctx(tx)
    .await?;

    Otp::delete_by_id(&t.id).exec(tx).await?;

    h.on_org_invitation_resolve(ctx, &uir).await?;

    uir.into_gql(ctx).await
}
