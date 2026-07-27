#[path = "./setup.rs"]
mod setup;
use setup::*;

async fn create_otp(tmp: &TmpDb, ty: &str, email: &str, otp: &str, secret: &str) -> Res<OtpSql> {
    let (otp_salt, otp_hashed) = rand_utils::otp_hash(otp)?;
    am_create!(Otp {
        ty: ty.to_owned(),
        email: email.to_owned(),
        secret_hashed: rand_utils::secret_hash(secret),
        otp_salt,
        otp_hashed,
        data: json!({
            "marker": "resolved",
        }),
    })
    .exec_without_ctx(&tmp.db)
    .await
}

// ---------------------------------------------------------------------------
// auth_otp_ensure_resolve
// ---------------------------------------------------------------------------

#[tokio::test]
async fn resolve_succeeds_and_resets_attempt() -> Res<()> {
    let d = setup().await?;
    let t = create_otp(&d.tmp, "register", "olivia@example.com", "123456", "sekret").await?;

    // Bump the attempt count up first, a successful resolve should reset it.
    Otp::update_many()
        .filter_by_id(&t.id)
        .col_expr(OtpColumn::TotalAttempt, Expr::value(3))
        .exec(&d.tmp.db)
        .await?;

    let s = d.s.data(d.h).finish();
    let q = "
    mutation($ty: String!, $id: String!, $secret: String!, $otp: String!) {
        otpResolveTest(ty: $ty, id: $id, secret: $secret, otp: $otp)
    }
    ";
    let v = value!({
        "ty": "register",
        "id": t.id,
        "secret": "sekret",
        "otp": "123456",
    });
    let expected = value!({
        "otpResolveTest": "resolved",
    });
    exec_assert(&s, q, Some(v), &expected).await;

    let after = Otp::find_by_id(&t.id).one_or_404(&d.tmp.db).await?;
    pretty_eq!(
        after.total_attempt,
        0,
        "attempt count should reset to 0 after a successful resolve"
    );

    d.tmp.drop().await
}

// increment goes through ctx.db() (see AuthOtpImpl), the raw
// pooled connection, rather than ctx.tx(), so this increment persists even
// though the overall request errors and its own transaction (see
// GrandLineData::cleanup) rolls back everything written through ctx.tx().
// Without this, attempt limiting could never take effect: every failed
// guess would have its increment undone by the same request's own rollback.
#[tokio::test]
async fn resolve_wrong_otp_increments_attempt_and_fails() -> Res<()> {
    let d = setup().await?;
    let t = create_otp(&d.tmp, "register", "olivia@example.com", "123456", "sekret").await?;

    let s = d.s.data(d.h).finish();
    let q = "
    mutation($ty: String!, $id: String!, $secret: String!, $otp: String!) {
        otpResolveTest(ty: $ty, id: $id, secret: $secret, otp: $otp)
    }
    ";
    let v = value!({
        "ty": "register",
        "id": t.id,
        "secret": "sekret",
        "otp": "000000",
    });
    exec_assert_err(&s, q, Some(v), &AuthErr::OtpResolveInvalid).await?;

    let after = Otp::find_by_id(&t.id).one_or_404(&d.tmp.db).await?;
    pretty_eq!(
        after.total_attempt,
        1,
        "a failed resolve attempt should still increment the counter, even though the request itself errors",
    );

    d.tmp.drop().await
}

#[tokio::test]
async fn resolve_wrong_secret_fails() -> Res<()> {
    let d = setup().await?;
    let t = create_otp(&d.tmp, "register", "olivia@example.com", "123456", "sekret").await?;

    let s = d.s.data(d.h).finish();
    let q = "
    mutation($ty: String!, $id: String!, $secret: String!, $otp: String!) {
        otpResolveTest(ty: $ty, id: $id, secret: $secret, otp: $otp)
    }
    ";
    let v = value!({
        "ty": "register",
        "id": t.id,
        "secret": "wrong-secret",
        "otp": "123456",
    });
    exec_assert_err(&s, q, Some(v), &AuthErr::OtpResolveInvalid).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn resolve_over_max_attempt_fails() -> Res<()> {
    let d = setup().await?;
    let t = create_otp(&d.tmp, "register", "olivia@example.com", "123456", "sekret").await?;

    let max = AuthConfig::default().otp_max_attempt;
    Otp::update_many()
        .filter_by_id(&t.id)
        .col_expr(OtpColumn::TotalAttempt, Expr::value(max))
        .exec(&d.tmp.db)
        .await?;

    let s = d.s.data(d.h).finish();
    let q = "
    mutation($ty: String!, $id: String!, $secret: String!,$otp: String!) {
        otpResolveTest(ty: $ty,id: $id,secret: $secret,otp: $otp)
    }
    ";
    let v = value!({
        "ty": "register",
        "id": t.id,
        "secret": "sekret",
        "otp": "123456",
    });
    exec_assert_err(&s, q, Some(v), &AuthErr::OtpResolveInvalid).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn resolve_expired_fails() -> Res<()> {
    let d = setup().await?;
    let t = create_otp(&d.tmp, "register", "olivia@example.com", "123456", "sekret").await?;

    let expires_ms = AuthConfig::default().otp_expires_ms;
    Otp::update_many()
        .filter_by_id(&t.id)
        .col_expr(
            OtpColumn::CreatedAt,
            Expr::value(now() - duration_ms(expires_ms) - duration_ms(1000)),
        )
        .exec(&d.tmp.db)
        .await?;

    let s = d.s.data(d.h).finish();
    let q = "
    mutation($ty: String!, $id: String!, $secret: String!, $otp: String!) {
        otpResolveTest(ty: $ty, id: $id, secret: $secret, otp: $otp)
    }
    ";
    let v = value!({
        "ty": "register",
        "id": t.id,
        "secret": "sekret",
        "otp": "123456",
    });
    exec_assert_err(&s, q, Some(v), &AuthErr::OtpResolveInvalid).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn resolve_unknown_id_fails() -> Res<()> {
    let d = setup().await?;

    let s = d.s.data(d.h).finish();
    let q = "
    mutation($ty: String!, $id: String!, $secret: String!, $otp: String!) {
        otpResolveTest(ty: $ty, id: $id, secret: $secret, otp: $otp)
    }
    ";
    let v = value!({
        "ty": "register",
        "id": ulid(),
        "secret": "sekret",
        "otp": "123456",
    });
    exec_assert_err(&s, q, Some(v), &AuthErr::OtpResolveInvalid).await?;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// auth_otp_ensure_re_request
// ---------------------------------------------------------------------------

#[tokio::test]
async fn re_request_no_row_is_noop() -> Res<()> {
    let d = setup().await?;

    let s = d.s.data(d.h).finish();
    let q = "
    mutation($ty: String!, $email: String!) {
        otpReRequestTest(ty: $ty, email: $email)
    }
    ";
    let v = value!({
        "ty": "register",
        "email": "no-such-row@example.com",
    });
    let expected = value!({
        "otpReRequestTest": true,
    });
    exec_assert(&s, q, Some(v), &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn re_request_within_cooldown_fails() -> Res<()> {
    let d = setup().await?;
    create_otp(&d.tmp, "register", "olivia@example.com", "123456", "sekret").await?;

    let s = d.s.data(d.h).finish();
    let q = "
    mutation($ty: String!, $email: String!) {
        otpReRequestTest(ty: $ty, email: $email)
    }
    ";
    let v = value!({
        "ty": "register",
        "email": "olivia@example.com",
    });
    exec_assert_err(&s, q, Some(v), &AuthErr::OtpReRequestTooSoon).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn re_request_past_cooldown_deletes_stale_row() -> Res<()> {
    let d = setup().await?;
    let t = create_otp(&d.tmp, "register", "olivia@example.com", "123456", "sekret").await?;

    let re_request_ms = AuthConfig::default().otp_re_request_ms;
    Otp::update_many()
        .filter_by_id(&t.id)
        .col_expr(
            OtpColumn::CreatedAt,
            Expr::value(now() - duration_ms(re_request_ms) - duration_ms(1000)),
        )
        .exec(&d.tmp.db)
        .await?;

    let s = d.s.data(d.h).finish();
    let q = "
    mutation($ty: String!, $email: String!) {
        otpReRequestTest(ty: $ty, email: $email)
    }
    ";
    let v = value!({
        "ty": "register",
        "email": "olivia@example.com",
    });
    let expected = value!({
        "otpReRequestTest": true,
    });
    exec_assert(&s, q, Some(v), &expected).await;

    let exists = Otp::find_by_id(&t.id).exists(&d.tmp.db).await?;
    pretty_eq!(
        exists,
        false,
        "the stale otp row should be deleted once its cooldown has passed"
    );

    d.tmp.drop().await
}
