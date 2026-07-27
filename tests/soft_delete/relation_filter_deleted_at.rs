#[path = "./setup.rs"]
mod setup;
use setup::*;

// _some/_none relation filters (see tests/relationship/relation_filter.rs)
// exclude soft-deleted related rows by default, same as any other resolver.
#[tokio::test]
async fn relation_some_excludes_soft_deleted_related_row_by_default() -> Res<()> {
    let d = setup().await?;

    let q = r#"
    query test {
        userSearch(
            filter: { aliases_some: { name: "Fauxlivia" } },
        ) {
            name
        }
    }
    "#;
    let expected = value!({
        "userSearch": [],
    });
    exec_assert(&d.s, q, None, &expected).await;

    d.tmp.drop().await
}

// Referencing deletedAt inside the nested _some filter opts that relation's
// subquery into seeing soft-deleted rows too, same has_deleted_at mechanism
// as a top-level filter, scoped to just this one relation's subquery.
#[tokio::test]
async fn relation_some_with_deleted_at_reference_sees_soft_deleted_related_row() -> Res<()> {
    let d = setup().await?;

    let q = r#"
    query test {
        userSearch(
            filter: { aliases_some: { name: "Fauxlivia", deletedAt_ne: null } },
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

// _none is the negation of _some, so once the nested filter's deletedAt
// reference makes the soft-deleted alias visible, _none correctly flips to
// false for a user that actually has one.
#[tokio::test]
async fn relation_none_with_deleted_at_reference_accounts_for_soft_deleted_related_row() -> Res<()> {
    let d = setup().await?;

    // Without a deletedAt reference, only the live "Liv" alias is visible to
    // the subquery, and it doesn't match "Fauxlivia", so _none is (vacuously,
    // from this subquery's point of view) true.
    let q = r#"
    query test {
        userSearch(
            filter: { aliases_none: { name: "Fauxlivia" } },
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

    // Once deletedAt is referenced, the subquery sees the soft-deleted
    // "Fauxlivia" alias too, so _none (no alias satisfies the nested filter)
    // is now false for Olivia.
    let q = r#"
    query test {
        userSearch(
            filter: { aliases_none: { name: "Fauxlivia", deletedAt_ne: null } },
        ) {
            name
        }
    }
    "#;
    let expected = value!({
        "userSearch": [],
    });
    exec_assert(&d.s, q, None, &expected).await;

    d.tmp.drop().await
}
