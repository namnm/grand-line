use crate::prelude::*;

#[derive(FromQueryResult)]
pub struct OrgMinimal {
    pub id: String,
}

pub struct AuthzCacheItem {
    pub role: RoleSql,
    pub org: Option<Arc<OrgMinimal>>,
}

/// Per-request cache for authz results.
pub type AuthzCache = Mutex<HashMap<String, Option<Arc<AuthzCacheItem>>>>;

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
