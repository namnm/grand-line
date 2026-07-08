use crate::prelude::*;

/// Resolves every pending org invitation matching user_id's email into a
/// UserInRole, deleting each consumed Otp row. No id/secret/otp challenge is
/// needed here (unlike org_invitation_accept_impl) because the caller already
/// proved ownership of the email through their own registration flow.
///
/// Intended to be called by a host app from within its own
/// AuthHandlers::on_register_resolve hook, chaining a fresh registration
/// straight into any orgs the new user was already invited to, e.g.
///
/// async fn on_register_resolve(&self, ctx: &Context<'_>, user_id: &str, ls: &LoginSessionSql) -> Res<()> {
///     org_invitation_resolve_by_email::<MyUser>(ctx, user_id).await?,
///     Ok(())
/// }
pub async fn org_invitation_resolve_by_email<U>(ctx: &Context<'_>, user_id: &str) -> Res<Vec<UserInRoleGql>>
where
    U: AuthUser,
{
    let tx = &*ctx.tx().await?;
    let h = &ctx.authz_config().handlers;

    let u = U::find().filter(U::col_id().eq(user_id)).one_or_404(tx).await?;
    let email = U::get_email(&u);

    let invitations = Otp::find()
        .include_deleted(false)
        .filter(OtpColumn::Ty.eq(OTP_TY_ORG_INVITATION))
        .filter(OtpColumn::Email.eq(email))
        .all(tx)
        .await?;

    let mut result = vec![];
    for inv in invitations {
        let d = OtpDataOrgInvitation::from_json(inv.data.clone())?;

        let uir = am_create!(UserInRole {
            user_id: user_id.to_owned(),
            role_id: d.role_id,
            org_id: Some(d.org_id),
        })
        .exec_without_ctx(tx)
        .await?;

        Otp::delete_by_id(&inv.id).exec(tx).await?;
        h.on_org_invitation_resolve(ctx, &uir).await?;

        result.push(uir.into_gql(ctx).await?);
    }

    Ok(result)
}
