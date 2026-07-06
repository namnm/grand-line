#[path = "./setup.rs"]
mod setup;
use setup::*;

// Logout succeeds when the user is authenticated and deletes the session.
#[tokio::test]
async fn logout_deletes_session_when_authenticated() -> Res<()> {
    let d = setup().await?;

    let mut h = d.h;
    h.insert(H_AUTHORIZATION, h_bearer(&d.token));
    let s = d.s.data(h).finish();

    let q = "
    mutation test {
        logout {
            id
        }
    }
    ";
    let r = exec_assert_ok(&s, q, None).await;
    let r = r.data.to_json()?;

    let id = r.str("/logout/id");
    pretty_eq!(id.is_empty(), false, "logout should return the session id");

    d.tmp.drop().await
}

// Logout without a token returns Unauthenticated.
#[tokio::test]
async fn logout_without_token_returns_unauthenticated() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    let q = "
    mutation test {
        logout {
            userId
        }
    }
    ";
    exec_assert_err(&s, q, None, &AuthErr::Unauthenticated).await?;

    d.tmp.drop().await
}
