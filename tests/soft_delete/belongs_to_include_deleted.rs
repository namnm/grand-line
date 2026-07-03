#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn belongs_to_include_deleted_returns_soft_deleted() -> Res<()> {
    let d = setup().await?;

    let q = "
    query test($id: ID!) {
        personDetail(id: $id) {
            user(includeDeleted: true) {
                name
            }
        }
    }
    ";
    let expected = value!({
        "personDetail": {
            "user": {
                "name": "Peter",
            },
        },
    });
    exec_assert_id(&d.s, q, &d.pid2, &expected).await;

    d.tmp.drop().await
}
