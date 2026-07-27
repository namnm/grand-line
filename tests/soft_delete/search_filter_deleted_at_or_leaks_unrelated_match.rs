#[path = "./setup.rs"]
mod setup;
use setup::*;

// An OR branch that references deletedAt opts the whole query into seeing
// soft-deleted rows, even ones matched only by a sibling OR branch that has
// nothing to do with deletion, this is intentional (see docs/filtering-sorting.md).
#[tokio::test]
async fn or_branch_referencing_deleted_at_also_surfaces_unrelated_deleted_rows() -> Res<()> {
    let d = setup().await?;

    let q = r#"
    query test {
        userSearch(
            filter: { OR: [{ deletedAt_ne: null }, { name: "Olivia" }] },
            orderBy: [NameAsc],
        ) {
            name
        }
    }
    "#;
    let expected = value!({
        "userSearch": [{
            "name": "Olivia",
        }, {
            "name": "Peter",
        }],
    });
    exec_assert(&d.s, q, None, &expected).await;

    d.tmp.drop().await
}

// Without any deletedAt reference at all, the same "name" branch alone stays
// scoped to live rows only, confirming the leak above is driven specifically
// by the sibling branch mentioning deletedAt, not by using OR itself.
#[tokio::test]
async fn or_without_deleted_at_reference_stays_scoped_to_live_rows() -> Res<()> {
    let d = setup().await?;

    let q = r#"
    query test {
        userSearch(
            filter: { OR: [{ name: "Olivia" }, { name: "Peter" }] },
            orderBy: [NameAsc],
        ) {
            name
        }
    }
    "#;
    let expected = value!({
        "userSearch": [{
            "name": "Olivia",
        }],
    });
    exec_assert(&d.s, q, None, &expected).await;

    d.tmp.drop().await
}
