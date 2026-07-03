#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn has_many_include_deleted_returns_all() -> Res<()> {
    let d = setup().await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            aliases(orderBy: [NameAsc], includeDeleted: true) {
                name
            }
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "aliases": [{
                "name": "Fauxlivia",
            }, {
                "name": "Liv",
            }],
        },
    });
    exec_assert_id(&d.s, q, &d.id1, &expected).await;

    d.tmp.drop().await
}
