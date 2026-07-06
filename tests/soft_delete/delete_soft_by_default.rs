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

    let Some(u) = User::find_by_id(&d.id1).one(&d.tmp.db).await? else {
        return TestErr::expect("user should still exist after soft delete");
    };

    pretty_eq!(u.deleted_at.is_some(), true, "user should be soft deleted by default");

    d.tmp.drop().await
}
