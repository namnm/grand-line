#[path = "./setup.rs"]
mod setup;
use setup::*;

// has_deleted_at's recursive traversal also descends into a NOT branch,
// `{ NOT: { deletedAt: null } }` is semantically deletedAt IS NOT NULL, same
// as deletedAt_ne: null, and should trigger the same include-deleted opt-in.
#[tokio::test]
async fn not_branch_referencing_deleted_at_also_triggers_include_deleted() -> Res<()> {
    let d = setup().await?;

    let q = "
    query test {
        userSearch(
            filter: { NOT: { deletedAt: null } },
        ) {
            name
        }
    }
    ";
    let expected = value!({
        "userSearch": [{
            "name": "Peter",
        }],
    });
    exec_assert(&d.s, q, None, &expected).await;

    d.tmp.drop().await
}

// Deeply nested (AND containing an OR containing the deletedAt reference)
// still gets detected, confirming the recursion isn't limited to one level.
#[tokio::test]
async fn deeply_nested_deleted_at_reference_is_still_detected() -> Res<()> {
    let d = setup().await?;

    let q = r#"
    query test {
        userSearch(
            filter: {
                AND: [
                    { OR: [{ deletedAt_ne: null }, { name: "Olivia" }] },
                    { name_ne: "someone-who-does-not-exist" },
                ],
            },
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
