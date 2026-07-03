use crate::prelude::*;

#[async_trait]
pub trait AuthzCacheContext<'a>
where
    Self: AuthContext<'a> + AuthzHttpContext<'a> + AuthzColContext<'a>,
{
    async fn authz_with_cache(&self, check: AuthzEnsure) -> Res<Option<Arc<AuthzCacheItem>>> {
        let field = self.field_impl();
        let name = field.name();
        let alias = field.alias();
        let has_alias = alias.is_some();
        let alias = alias.unwrap_or(name).to_owned();

        // Collect alias->schema path entries for the entire selection subtree
        // synchronously before any .await so SelectionField<'_> does not cross
        // an async boundary.
        let mut path_entries: Vec<(String, String)> = if has_alias {
            vec![(alias.clone(), name.to_owned())]
        } else {
            vec![]
        };
        collect_alias_recursively(field, name, &alias, has_alias, &mut path_entries);

        let m = self.authz_cache_or_init().await?;
        let mut guard = m.lock().await;
        if let Some(v) = guard.get(&alias) {
            let v = v.clone();
            drop(guard);
            return Ok(v);
        }

        let v = self.authz_without_cache(check).await?.map(Arc::new);
        guard.insert(alias.clone(), v.clone());
        drop(guard);

        // Store the path map so any nested resolver can translate its alias-based
        // path to a schema path for row policy lookup. or_insert keeps the first
        // mapping in case two root fields share overlapping response keys.
        let pm = self.authz_path_map_or_init().await?;
        let mut pm_guard = pm.0.lock().await;
        // need to use for loop to keep existing keys from other operations.
        for (a, n) in path_entries {
            pm_guard.entry(a).or_insert(n);
        }
        drop(pm_guard);

        Ok(v)
    }

    async fn authz_without_cache(&self, check: AuthzEnsure) -> Res<Option<AuthzCacheItem>> {
        let k = self.authz_config().role_id_header_key;
        let role_id = self.get_header(k)?.trim().to_owned();
        if role_id.is_empty() {
            return Err(MyErr::HeaderRoleId404.into());
        }

        let mut q = Role::find()
            .include_deleted(false)
            .filter_by_id(&role_id)
            .filter(RoleColumn::Realm.eq(&check.realm));

        let org = if check.org {
            let o = self.org_unchecked().await?;
            q = q.filter(RoleColumn::OrgId.eq(&o.id));
            Some(o)
        } else {
            q = q.filter(RoleColumn::OrgId.is_null());
            None
        };

        if check.user {
            let user_id = self.auth().await?;
            let mut sub = UserInRole::find()
                .include_deleted(false)
                .select_only()
                .column(UserInRoleColumn::RoleId)
                .filter(UserInRoleColumn::UserId.eq(user_id));
            if check.org {
                let o = self.org_unchecked().await?;
                sub = sub.filter(UserInRoleColumn::OrgId.eq(&o.id));
            } else {
                sub = sub.filter(UserInRoleColumn::OrgId.is_null());
            }
            q = q.filter(RoleColumn::Id.in_subquery(sub.into_query()));
        }

        let tx = &*self.tx().await?;
        let role = q.one(tx).await?;

        let Some(role) = role else {
            return Ok(None);
        };

        let map = ColPolicy::from_json(role.col_policy.clone())?;
        if let Some(p) = map.get("*").or_else(|| map.get(self.field_impl().name()))
            && self.authz_col_check_inputs(&p.inputs)
            && self.authz_col_check_output(&p.output)
        {
            let c = AuthzCacheItem {
                role,
                org,
            };
            return Ok(Some(c));
        }

        Ok(None)
    }

    async fn authz_cache_or_init(&self) -> Res<Arc<AuthzCache>> {
        self.cache(async || Ok(Mutex::new(HashMap::new()))).await
    }

    async fn authz_path_map_or_init(&self) -> Res<Arc<AuthzPathMap>> {
        self.cache(async || Ok(AuthzPathMap(Mutex::new(HashMap::new())))).await
    }

    /// Translate the current resolver's alias-based path to schema field names.
    /// The root resolver pre-builds the full alias->schema path map for its entire
    /// selection subtree, so this is a single O(1) lookup with no per-level registration.
    async fn authz_row_field_path(&self) -> Res<String> {
        let alias_path = self.field_path_without_number_index();
        let pm = self.authz_path_map_or_init().await?;
        let guard = pm.0.lock().await;
        let p = guard.get(&alias_path).cloned().unwrap_or(alias_path);
        drop(guard);
        Ok(p)
    }

    /// Return the root resolver's cache key from the current resolver's path.
    /// Because #[authz] is only allowed on root resolvers, the cache only ever
    /// holds single-segment keys. Nested resolvers just need the first non-numeric
    /// path segment (the root alias or field name) and verify it is in the cache.
    async fn authz_cache_key(&self) -> Res<String> {
        let alias = if let Some(node) = self.path_node_impl() {
            node.to_string_vec().first().cloned()
        } else {
            None
        }
        .unwrap_or_else(|| {
            let field = self.field_impl();
            field.alias().unwrap_or_else(|| field.name()).to_owned()
        });

        let m = self.authz_cache_or_init().await?;
        let guard = m.lock().await;
        if guard.contains_key(&alias) {
            drop(guard);
            return Ok(alias);
        }

        drop(guard);
        Err(MyErr::MissingMacro.into())
    }
}

#[async_trait]
impl<'a> AuthzCacheContext<'a> for Context<'a> {
}

fn collect_alias_recursively(
    parent: SelectionField<'_>,
    name_path: &str,
    alias_path: &str,
    has_alias: bool,
    out: &mut Vec<(String, String)>,
) {
    for child in parent.selection_set() {
        let name = child.name();
        let name_path = format!("{name_path}.{name}");
        let alias = child.alias();
        let has_alias = has_alias || alias.is_some();
        let alias = alias.unwrap_or(name);
        let alias_path = format!("{alias_path}.{alias}");
        if has_alias {
            out.push((alias_path.clone(), name_path.clone()));
        }
        collect_alias_recursively(child, &name_path, &alias_path, has_alias, out);
    }
}
