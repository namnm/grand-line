#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn count_filters_by_deleted_at() -> Res<()> {
    let d = setup().await?;

    let q = "
    query test($filter: UserFilter) {
        userCount(
            filter: $filter,
        )
    }
    ";
    let v = value!({
        "filter": { "deletedAt_ne": null },
    });
    let expected = value!({
        "userCount": 1,
    });
    exec_assert(&d.s, q, Some(v), &expected).await;

    let v = value!({
        "filter": include_all_filter(),
    });
    let expected = value!({
        "userCount": 2,
    });
    exec_assert(&d.s, q, Some(v), &expected).await;

    d.tmp.drop().await
}
