#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn logout_ends_the_current_session() -> Res<()> {
    let d = setup().await?;
    let (_, bearer) = register_and_resolve(&d, "astrid@fbi.example", "Farnsworth-Field-Agent-88!", "884421").await?;

    let mut h = d.h.clone();
    h.insert(H_AUTHORIZATION, bearer.clone());
    let s = d.schema(h);
    let q = "
    mutation {
        logout {
            id
        }
    }
    ";
    exec_assert_ok(&s, q, None).await;

    let mut h = d.h.clone();
    h.insert(H_AUTHORIZATION, bearer);
    let s = d.schema(h);
    let q = "
    query {
        loginSessionCurrent {
            userId
        }
    }
    ";
    let expected = value!({
        "loginSessionCurrent": null,
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}
