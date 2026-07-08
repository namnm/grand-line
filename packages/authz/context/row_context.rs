use crate::prelude::*;
use serde::de::DeserializeOwned;

// Build "users.posts" style path from the current resolver position,
// skipping list indices so "users.0.posts" becomes "users.posts".
fn authz_field_path(ctx: &Context<'_>) -> String {
    let Some(node) = ctx.path_node.as_ref() else {
        return ctx.field().name().to_string();
    };
    node.to_string_vec()
        .into_iter()
        .filter(|s| s.parse::<usize>().is_err())
        .collect::<Vec<_>>()
        .join(".")
}

/// Retrieve the row-level filter defined in the current operation's policy.
///
/// Must be called after `authz_ensure_in_macro` has run (i.e. inside a resolver
/// that has the #[authz(...)] attribute). Returns None when the matched policy
/// has no row script, meaning all rows are accessible.
#[async_trait]
pub trait AuthzRowContext {
    async fn authz_row<F>(&self) -> Res<Option<F>>
    where
        F: Sized + Serialize + DeserializeOwned;
}

#[async_trait]
impl AuthzRowContext for Context<'_> {
    async fn authz_row<F>(&self) -> Res<Option<F>>
    where
        F: Sized + Serialize + DeserializeOwned,
    {
        let cache_k = self.authz_cache_key().await?;
        let cache_m = self.authz_cache_or_init().await?;
        let guard = cache_m.lock().await;
        let Some(cached) = guard.get(&cache_k).cloned() else {
            return Err(MyErr::MissingMacro.into());
        };
        drop(guard);

        let Some(item) = cached.as_ref() else {
            return Err(MyErr::Unauthorized.into());
        };

        let path = authz_field_path(self);
        let field = self.field().name();
        let map = PolicyOperations::from_json(item.role.operations.clone())?;
        let Some(p) = map.get(&path).or_else(|| map.get(field)).or_else(|| map.get("*")) else {
            return Err(MyErr::Unauthorized.into());
        };

        let Some(script) = &p.row else {
            return Ok(None);
        };

        // auth/authz failures become None scope vars so scripts that don't
        // reference current_user / current_org still work in those realms
        let user_id = self.auth().await.ok();
        let org_id = item.org.as_ref().map(|o| o.id.as_str());

        let h = &self.authz_config().handlers;

        let Some(json) = h.on_formula(self).await? else {
            return Ok(None);
        };
        let filter = F::from_json(json).map_err(|e| MyErr::RowScript(e.to_string()))?;
        Ok(Some(filter))
    }
}
