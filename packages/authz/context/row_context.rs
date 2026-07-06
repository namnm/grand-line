use crate::prelude::*;

#[async_trait]
pub trait AuthzRowContext<'a>
where
    Self: AuthzConfigContext<'a> + AuthzRoleContext<'a>,
{
    /// Retrieve the row-level filter defined in the current operation's policy.
    /// Results are cached per (filter type, field path) for the lifetime of the request.
    async fn authz_row<F>(&self) -> Res<Option<F>>
    where
        F: DeserializeOwned + Clone + Send + Sync + 'static,
    {
        let path = self.authz_row_field_path().await?;
        let k = (TypeId::of::<F>(), path.clone());

        let cache = self.authz_row_cache_or_init().await?;
        let guard = cache.0.lock().await;
        if let Some(cached) = guard.get(&k) {
            let v = Arc::clone(cached)
                .downcast::<Option<F>>()
                .map_err(|_| MyErr::RowCacheDowncast)?;
            drop(guard);
            return Ok((*v).clone());
        }
        drop(guard);

        let r = self.authz_row_get_filter::<F>(&path).await?;
        cache.0.lock().await.insert(k, Arc::new(r.clone()) as ArcAny);

        Ok(r)
    }

    /// Get dsl script from the role row policy, execute it to get json and deserialize into target filter type.
    async fn authz_row_get_filter<F>(&self, path: &str) -> Res<Option<F>>
    where
        F: DeserializeOwned,
    {
        let r = self.authz_role().await?;
        let Some(script) = r.as_ref().role.row_policy.get(path) else {
            return Ok(None);
        };
        if script.is_null() {
            return Ok(None);
        }
        let Some(script) = script.as_str() else {
            return Err(MyErr::RowScript404.into());
        };
        let h = &self.authz_config().handlers;
        let Some(json) = self.authz_execute_script(h, script).await? else {
            return Ok(None);
        };
        let f = F::from_json(json)?;
        Ok(Some(f))
    }

    /// Get or create cache for authz row.
    async fn authz_row_cache_or_init(&self) -> Res<Arc<AuthzRowCache>> {
        self.cache(async || Ok(AuthzRowCache(Mutex::new(HashMap::new())))).await
    }

    /// Helper to execute the dsl script using authz handler from trait definition.
    async fn authz_execute_script(&self, h: &Arc<dyn AuthzHandlers>, script: &str) -> Res<Option<JsonValue>>;

    /// Similar to authz_row but do not return error if not in authz macro.
    /// To make it graceful and can be used in relationship without root authz macro.
    async fn authz_row_graceful<F>(&self) -> Res<Option<F>>
    where
        F: DeserializeOwned + Clone + Send + Sync + 'static,
    {
        match self.authz_row::<F>().await {
            Err(e) if e.0.code() == MyErr::MissingMacro.code() => Ok(None),
            f => f,
        }
    }
}

#[async_trait]
impl<'a> AuthzRowContext<'a> for Context<'a> {
    async fn authz_execute_script(&self, h: &Arc<dyn AuthzHandlers>, script: &str) -> Res<Option<JsonValue>> {
        h.execute_script(self, script).await
    }
}
