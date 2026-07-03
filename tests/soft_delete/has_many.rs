#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn has_many_excludes_soft_deleted_by_default() -> Res<()> {
    let d = setup().await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            aliases {
                name
            }
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "aliases": [{
                "name": "Liv",
            }],
        },
    });
    exec_assert_id(&d.s, q, &d.id1, &expected).await;

    d.tmp.drop().await
}
