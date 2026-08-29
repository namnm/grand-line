use crate::prelude::*;

/// Row-level policy filter resolution: runs the role's row policy dsl script
/// for the current field path and caches the result for the request.
#[async_trait]
pub trait AuthzRowContext<'a>
where
    Self: AuthzConfigContext<'a> + AuthzRoleContext<'a>,
{
    /// Retrieve the row-level filter defined in the current operation's policy.
    /// Results are cached per (filter type, field path) for the lifetime of the request.
    async fn authz_row<F>(&self) -> Res<Option<F>>
    where
        F: FilterKeys + DeserializeOwned + Clone + Send + Sync + 'static,
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
        F: FilterKeys + DeserializeOwned,
    {
        let r = self.authz_role().await?;
        let Some(script) = r.row_policy.get(path) else {
            return Ok(None);
        };
        // No policy entry and a policy entry whose handler declines are opposites
        // on a security boundary. The first is "no filter for this path", the
        // second is "a policy is configured but nothing handled it", which must
        // deny instead of failing open. allow_unhandled_row_policy restores the
        // lenient integration mode, see AuthzConfig.
        let h = &self.authz_config().handlers;
        let Some(json) = self.authz_execute_script(h, script).await? else {
            if self.authz_config().allow_unhandled_row_policy {
                return Ok(None);
            }
            return Err(MyErr::RowPolicyUnhandled.into());
        };
        Self::authz_row_strict_filter::<F>(json).map(Some)
    }

    /// Deserialize a filter arriving from the authorization boundary, strictly.
    /// The generated filters deserialize leniently (#[serde(default), unknown
    /// keys silently dropped) because client input is coerced by graphql first.
    /// Here a typo in policy data would otherwise deserialize into an empty
    /// filter, i.e. an empty WHERE, i.e. every row, invisible from both sides.
    /// So every key must map to a known field of F, and an empty object is an
    /// error rather than an unrestricted query.
    fn authz_row_strict_filter<F>(json: JsonValue) -> Res<F>
    where
        F: FilterKeys + DeserializeOwned,
    {
        Self::authz_row_strict_keys(&json, F::known_keys())?;
        F::from_json(json)
    }

    /// Validate every key of one filter object, recursing through the and/or/not
    /// nesting a filter allows. Those branches hold the same filter type, so the
    /// same key set applies, and checking only the top level would leave the very
    /// same typo silently dropped one level down.
    fn authz_row_strict_keys(json: &JsonValue, keys: &[&'static str]) -> Res<()> {
        let JsonValue::Object(m) = json else {
            // a non-object payload fails deserialization with a serde error
            return Ok(());
        };
        if m.is_empty() {
            return Err(MyErr::RowPolicyFilterEmpty.into());
        }
        for (k, v) in m {
            if !keys.contains(&k.as_str()) {
                return Err(MyErr::RowPolicyFilterKey {
                    k: k.to_owned(),
                }
                .into());
            }
            match k.as_str() {
                "and" | "or" => {
                    for v in v.as_array().into_iter().flatten() {
                        Self::authz_row_strict_keys(v, keys)?;
                    }
                }
                "not" => Self::authz_row_strict_keys(v, keys)?,
                _ => {}
            }
        }
        Ok(())
    }

    /// Get or create cache for authz row.
    async fn authz_row_cache_or_init(&self) -> Res<Arc<AuthzRowCache>> {
        self.cache(async || Ok(AuthzRowCache(Mutex::new(HashMap::new())))).await
    }

    /// Helper to execute the dsl script using authz handler from trait definition.
    async fn authz_execute_script(&self, h: &Arc<dyn AuthzHandlers>, script: &str) -> Res<Option<JsonValue>>;

    /// Similar to authz_row but do not return error when no authz guard ran.
    /// To make it graceful and can be used in relationship without a root guard.
    async fn authz_row_graceful<F>(&self) -> Res<Option<F>>
    where
        F: FilterKeys + DeserializeOwned + Clone + Send + Sync + 'static,
    {
        match self.authz_row::<F>().await {
            Err(e) if e.0.code() == MyErr::MissingGuard.code() => Ok(None),
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
