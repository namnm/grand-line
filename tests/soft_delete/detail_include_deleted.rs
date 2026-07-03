#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn detail_include_deleted_returns_soft_deleted() -> Res<()> {
    let d = setup().await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id, includeDeleted: true) {
            name
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "name": "Peter",
        },
    });
    exec_assert_id(&d.s, q, &d.id2, &expected).await;

    d.tmp.drop().await
}
