#[path = "./setup.rs"]
mod setup;
use setup::*;

// Contrast with the OR case: an implicit AND (plain top-level fields) that
// references deletedAt still opts the query into seeing deleted rows, but the
// condition itself narrows rather than widens, since every field in an AND
// must hold at once.
#[tokio::test]
async fn and_referencing_deleted_at_narrows_instead_of_leaking() -> Res<()> {
    let d = setup().await?;

    let q = "
    query test($name: String!) {
        userSearch(
            filter: { deletedAt_ne: null, name: $name },
        ) {
            name
        }
    }
    ";

    // Peter is both deleted and named Peter, matches.
    let v = value!({ "name": "Peter" });
    let expected = value!({
        "userSearch": [{
            "name": "Peter",
        }],
    });
    exec_assert(&d.s, q, Some(v), &expected).await;

    // Olivia is named Olivia but not deleted, the AND excludes her even
    // though deletedAt_ne is present in the filter tree.
    let v = value!({ "name": "Olivia" });
    let expected = value!({
        "userSearch": [],
    });
    exec_assert(&d.s, q, Some(v), &expected).await;

    d.tmp.drop().await
}
