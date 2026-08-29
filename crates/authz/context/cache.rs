use crate::prelude::*;

/// Minimal org projection loaded during authz checks, only the id is selected.
#[derive(FromQueryResult)]
pub struct OrgMinimal {
    pub id: String,
}

/// Cached result of a passed authz check: the matched role's id and parsed
/// row_policy (needed later by authz_row), and, if the check required org
/// scoping, the resolved org.
pub struct AuthzCacheItem {
    pub role_id: String,
    pub row_policy: RowPolicy,
    pub org: Option<Arc<OrgMinimal>>,
}

/// One cached authz result: the check that produced it plus the role it matched.
/// The check is part of the entry, not the key, so a resolver guarded by two
/// `authz_ensure` calls with different requirements evaluates both instead of
/// the second one silently reusing the first one's result.
pub type AuthzCacheEntry = (AuthzEnsure, Option<Arc<AuthzCacheItem>>);

/// Per-request cache for authz results, keyed by the root field's alias and
/// holding one entry per distinct check run for that field.
pub type AuthzCache = Mutex<HashMap<String, Vec<AuthzCacheEntry>>>;

/// Per-request cache for authz_row results, keyed by (filter TypeId, field path).
/// Avoids calling the handler repeatedly for the same field in the same request
/// (e.g. N parents each resolving the same has_one relation with row auth).
/// This seems to be a generic type, so we need to create a struct wrapper to avoid conflict.
pub struct AuthzRowCache(pub Mutex<HashMap<(TypeId, String), ArcAny>>);

/// Per-request flat map from alias-based path to schema-based path, built once
/// by the root resolver from its full selection tree. Covers N levels of nesting
/// regardless of which intermediate resolvers call authz_row.
/// Key: dot-joined alias segments (e.g. "pd.cmt"). Value: schema names (e.g. "postDetail.comments").
/// This seems to be a generic type, so we need to create a struct wrapper to avoid conflict.
pub struct AuthzPathMap(pub Mutex<HashMap<String, String>>);
