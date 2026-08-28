#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

use axum::http::HeaderMap;

#[path = "../_fixtures/common.rs"]
mod common;
pub use common::*;

/// Sets the acting org/role headers on top of a bearer, for check = authz_org calls.
pub fn h_authz(mut h: HeaderMap, org_id: &str, role_id: &str) -> HeaderMap {
    h.insert(H_ORG_ID, h_str(org_id));
    h.insert(H_ROLE_ID, h_str(role_id));
    h
}

/// Looks up the "System" system-realm role seeded by grand_line_examples_saas::seed.
pub async fn seeded_bootstrap_role(tmp: &TmpDb) -> Res<RoleSql> {
    Role::find()
        .filter(RoleColumn::Name.eq("System"))
        .one_or_404(&tmp.db)
        .await
}

/// The permissive col_policy/row_policy shape used across this suite, mirrors
/// the wildcard policy grand_line_examples_saas::seed gives the bootstrap admin.
pub fn wildcard_col_policy() -> GraphQLValue {
    value!({
        "*": {
            "inputs": {
                "allow": true,
                "children": {
                    "**": {
                        "allow": true,
                        "children": null,
                    },
                },
            },
            "output": {
                "allow": true,
                "children": {
                    "**": {
                        "allow": true,
                        "children": null,
                    },
                },
            },
        },
    })
}
