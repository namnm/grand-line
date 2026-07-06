#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn delete_permanent_removes_row_from_db() -> Res<()> {
    let d = setup().await?;

    let q = "
    mutation test($id: ID!) {
        userDelete(id: $id, permanent: true) {
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

    let exist = User::find_by_id(&d.id1).exists(&d.tmp.db).await?;
    pretty_eq!(exist, false, "user should be permanently deleted from db");

    d.tmp.drop().await
}
