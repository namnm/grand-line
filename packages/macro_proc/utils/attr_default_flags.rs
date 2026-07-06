pub const FEATURE_MODEL_CREATED_AT: bool = cfg!(feature = "model_created_at");
pub const FEATURE_MODEL_UPDATED_AT: bool = cfg!(feature = "model_updated_at");
pub const FEATURE_MODEL_DELETED_AT: bool = cfg!(feature = "model_deleted_at");
pub const FEATURE_MODEL_BY_ID: bool = cfg!(feature = "model_by_id");

pub const FEATURE_RESOLVER_TX: bool = cfg!(feature = "resolver_tx");
pub const FEATURE_RESOLVER_CTX: bool = cfg!(feature = "resolver_ctx");
pub const FEATURE_RESOLVER_INCLUDE_DELETED: bool = cfg!(feature = "resolver_include_deleted");
pub const FEATURE_RESOLVER_PERMANENT_DELETE: bool = cfg!(feature = "resolver_permanent_delete");
pub const FEATURE_RESOLVER_AUTHZ_ROW: bool = cfg!(feature = "resolver_authz_row");
pub const FEATURE_RESOLVER_RELATION_COUNT: bool = cfg!(feature = "resolver_relation_count");
