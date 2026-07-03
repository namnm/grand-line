#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn search_filters_by_deleted_at() -> Res<()> {
    let d = setup().await?;

    let q = "
    query test($filter: UserFilter) {
        userSearch(
            filter: $filter,
            orderBy: [NameAsc],
        ) {
            name
        }
    }
    ";
    let v = value!({
        "filter": { "deletedAt_ne": null },
    });
    let expected = value!({
        "userSearch": [{
            "name": "Peter",
        }],
    });
    exec_assert(&d.s, q, Some(v), &expected).await;

    let v = value!({
        "filter": include_all_filter(),
    });
    let expected = value!({
        "userSearch": [{
            "name": "Olivia",
        }, {
            "name": "Peter",
        }],
    });
    exec_assert(&d.s, q, Some(v), &expected).await;

    d.tmp.drop().await
}
