#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn many_to_many_excludes_soft_deleted_by_default() -> Res<()> {
    let d = setup().await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            orgs {
                name
            }
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "orgs": [{
                "name": "Fringe",
            }],
        },
    });
    exec_assert_id(&d.s, q, &d.id1, &expected).await;

    d.tmp.drop().await
}
