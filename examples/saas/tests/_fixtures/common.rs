use axum::http::{HeaderMap, HeaderValue};
pub use grand_line_examples_saas::prelude::*;
pub use grand_line_examples_saas::{AppSchema, schema as build_app_schema, seed as seed_app_db};

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

/// A fresh, isolated in-memory sqlite db seeded exactly like production (the
/// "system@example.com" bootstrap admin, its "System" role, and the
/// "Acme Inc" org), wired with the real Saas auth/authz implementations, same
/// as examples/saas's own main.rs, so these tests exercise the actual
/// production wiring end to end rather than a test double.
pub struct Setup {
    pub tmp: TmpDb,
    pub h: HeaderMap,
}

pub async fn setup() -> Res<Setup> {
    let tmp = tmp_db!(User, Org, Role, UserInRole, LoginSession, Otp, Impersonation);
    if let Err(e) = seed_app_db(&tmp.db).await {
        return TestErr::expect(&e.to_string());
    }

    let h = init_common_headers();

    Ok(Setup {
        tmp,
        h,
    })
}

impl Setup {
    /// Builds a finished schema wired with tmp's db and h, a fresh instance
    /// each call since a SchemaBuilder is consumed by finish(), needed
    /// because a single flow test moves through several different actors.
    pub fn schema(&self, h: HeaderMap) -> AppSchema {
        build_app_schema(&self.tmp.db).data(h).finish()
    }
}

// ---------------------------------------------------------------------------
// Header helpers
// ---------------------------------------------------------------------------

/// Builds an Authorization bearer header authenticating a login session,
/// matching how SaasAuthSessionImpl looks up LoginSession rows by id/secret.
pub fn h_bearer_for(id: &str, secret: &str) -> HeaderValue {
    let token = rand_utils::qs_token(id, secret).unwrap_or_default();
    h_bearer(&token)
}

// ---------------------------------------------------------------------------
// Response helpers
// ---------------------------------------------------------------------------

/// Converts a Response's data into plain JSON for JsonTestingHelper-style
/// pointer lookups, e.g. to pull a server-generated id out of a mutation's
/// response before using it in the next call of the same flow.
pub fn json_data(res: &Response) -> JsonValue {
    res.data.clone().into_json().unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Otp helpers
// ---------------------------------------------------------------------------

/// Overwrites otp_hashed on the Otp row identified by id so it resolves with
/// known_otp, simulating reading the one-time code off the real mailer (the
/// register/forgot/invitationCreate mutations only ever print it to stdout).
pub async fn known_otp_for(tmp: &TmpDb, otp_id: &str, known_otp: &str) -> Res<()> {
    let row = Otp::find_by_id(otp_id).one_or_404(&tmp.db).await?;
    let otp_hashed = rand_utils::otp_hash_with_salt(&row.otp_salt, known_otp)?;
    Otp::update_many()
        .filter_by_id(otp_id)
        .col_expr(OtpColumn::OtpHashed, Expr::value(otp_hashed))
        .exec(&tmp.db)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Real register -> registerResolve round trip
// ---------------------------------------------------------------------------

/// Registers and resolves a brand-new user via the real register/registerResolve
/// mutations end to end, overriding the otp hash to known_otp since the real
/// code is only ever "mailed" (see known_otp_for). Returns the created user's
/// id and a bearer header authenticating their fresh login session.
pub async fn register_and_resolve(
    d: &Setup,
    email: &str,
    password: &str,
    known_otp: &str,
) -> Res<(String, HeaderValue)> {
    let s = d.schema(d.h.clone());
    let q = "
    mutation($email: Email!, $password: String!) {
        register(data: { email: $email, password: $password }) {
            id
            secret
        }
    }
    ";
    let v = value!({
        "email": email,
        "password": password,
    });
    let res = exec_assert_ok(&s, q, Some(v)).await;
    let data = json_data(&res);
    let otp_id = data.str("/register/id").to_owned();
    let otp_secret = data.str("/register/secret").to_owned();

    known_otp_for(&d.tmp, &otp_id, known_otp).await?;

    let s = d.schema(d.h.clone());
    let q = "
    mutation($id: String!, $secret: String!, $otp: String!) {
        registerResolve(data: { id: $id, secret: $secret, otp: $otp }) {
            id
            secret
        }
    }
    ";
    let v = value!({
        "id": otp_id,
        "secret": otp_secret,
        "otp": known_otp,
    });
    let res = exec_assert_ok(&s, q, Some(v)).await;
    let data = json_data(&res);
    let session_id = data.str("/registerResolve/id").to_owned();
    let session_secret = data.str("/registerResolve/secret").to_owned();

    let user = User::find()
        .filter(UserColumn::Email.eq(email))
        .one_or_404(&d.tmp.db)
        .await?;

    Ok((user.id, h_bearer_for(&session_id, &session_secret)))
}

/// Logs in as an already-registered user via the real login mutation, returns
/// a bearer header authenticating the fresh session.
pub async fn login_bearer(d: &Setup, email: &str, password: &str) -> Res<HeaderValue> {
    let s = d.schema(d.h.clone());
    let q = "
    mutation($email: Email!, $password: String!) {
        login(data: { email: $email, password: $password }) {
            id
            secret
        }
    }
    ";
    let v = value!({
        "email": email,
        "password": password,
    });
    let res = exec_assert_ok(&s, q, Some(v)).await;
    let data = json_data(&res);
    let id = data.str("/login/id").to_owned();
    let secret = data.str("/login/secret").to_owned();
    Ok(h_bearer_for(&id, &secret))
}
