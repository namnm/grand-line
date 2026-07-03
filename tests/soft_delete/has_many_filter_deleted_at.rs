#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn has_many_filters_by_deleted_at() -> Res<()> {
    let d = setup().await?;

    let q = "
    query test($id: ID!, $filter: AliasFilter) {
        userDetail(id: $id) {
            aliases(
                filter: $filter,
                orderBy: [NameAsc],
            ) {
                name
            }
        }
    }
    ";
    let v = value!({
        "id": d.id1,
        "filter": { "deletedAt_ne": null },
    });
    let expected = value!({
        "userDetail": {
            "aliases": [{
                "name": "Fauxlivia",
            }],
        },
    });
    exec_assert(&d.s, q, Some(v), &expected).await;

    let v = value!({
        "id": d.id1,
        "filter": include_all_filter(),
    });
    let expected = value!({
        "userDetail": {
            "aliases": [{
                "name": "Fauxlivia",
            }, {
                "name": "Liv",
            }],
        },
    });
    exec_assert(&d.s, q, Some(v), &expected).await;

    d.tmp.drop().await
}
