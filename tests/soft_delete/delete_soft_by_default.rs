#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn delete_sets_deleted_at_by_default() -> Res<()> {
    let d = setup().await?;

    let q = "
    mutation test($id: ID!) {
        userDelete(id: $id) {
            id
        }
    }
    ";
    let expected = value!({
        "userDelete": {
            "id": d.id1,
        },
    });
    exec_assert_id(&d.s, q, &d.id1, &expected).await;

    match User::find_by_id(&d.id1).one(&d.tmp.db).await? {
        Some(u) => assert!(
            u.deleted_at.is_some(),
            "it should be soft delete by default, found deleted_at=None",
        ),
        None => assert!(
            false,
            "it should be soft delete by default, found None returned from db",
        ),
    }

    d.tmp.drop().await
}
