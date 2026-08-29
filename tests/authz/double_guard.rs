#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// A second authz guard on the same resolver must not be skipped
// ---------------------------------------------------------------------------

// A resolver guarded by authz_org and then authz_system: the two guards carry
// different realm/org/user requirements, and the authz cache is keyed per
// check, so the second one is evaluated on its own terms. role_id1 only
// carries an org realm role, so authz_system cannot match it and the request
// fails. When the cache was keyed by the root alias alone, the second guard
// silently inherited the first one's passing result and the resolver ran.
#[tokio::test]
async fn a_second_guard_with_different_requirements_is_not_satisfied_from_the_cache() -> Res<()> {
    let d = setup_with_col_wildcard().await?;
    let h = auth_headers(d.h, &d.org_id1, &d.user_id1, &d.role_id1);
    let s = d.s.data(h).finish();

    let q = "
    query {
        doubleGuardRejects
    }
    ";
    exec_assert_err(&s, q, None, &AuthzErr::Unauthorized).await?;

    d.tmp.drop().await
}

// The same check twice shares one lookup: the cache entry carries the check
// that produced it, so an equal check still hits.
#[tokio::test]
async fn the_same_check_twice_still_shares_one_lookup() -> Res<()> {
    let d = setup_with_col_wildcard().await?;
    let h = auth_headers(d.h, &d.org_id1, &d.user_id1, &d.role_id1);
    let s = d.s.data(h).finish();

    let q = "
    query {
        doubleGuardAccepts
    }
    ";
    let expected = value!({
        "doubleGuardAccepts": true,
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}
