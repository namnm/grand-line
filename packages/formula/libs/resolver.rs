use _core::prelude::*;
use chrono::Utc;
use rhai::Dynamic;

/// Context passed to each FormulaResolver::resolve call.
///
/// Deliberately minimal: no user_id/org_id (resolvers do not need the request
/// identity, current_user/current_org are pushed straight into the Rhai scope
/// by eval_formula instead) and no db transaction (row-formula db access goes
/// through the sync FormulaDbAccessor track, called from inside the script via
/// db_find_one / db_find_many, not through this async pre-fetch track).
pub struct FormulaCtx<'a> {
    /// BCP 47 locale tag for this eval (e.g. "en", "vi", "ja").
    pub locale: &'a str,
    /// Values already resolved by earlier nodes in the dependency graph.
    /// A resolver for node B that declares deps: ["a"] can read
    /// ctx.resolved["a"] and is guaranteed to find it present.
    pub resolved: &'a HashMap<String, Dynamic>,
}

/// Pre-fetch track: async resolver that injects a scope variable before eval.
///
/// The variable name is declared on FormulaDepNode, not here, resolve
/// receives that name so one struct can handle multiple names if needed.
#[async_trait]
pub trait FormulaResolver: Send + Sync {
    async fn resolve(&self, name: &str, ctx: &FormulaCtx<'_>) -> Res<Dynamic>;
}

/// Built-in resolver: exposes now as current UTC milliseconds since epoch.
pub struct NowResolver;

#[async_trait]
impl FormulaResolver for NowResolver {
    async fn resolve(&self, _name: &str, _ctx: &FormulaCtx<'_>) -> Res<Dynamic> {
        Ok(Dynamic::from(Utc::now().timestamp_millis()))
    }
}
