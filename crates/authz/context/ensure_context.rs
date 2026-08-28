use crate::prelude::*;

/// Authz check parameters for a single guard call.
pub struct AuthzEnsure {
    /// Role realm required, e.g. system, org, public.
    pub realm: String,
    /// Whether the check requires the role to be scoped to the request's org.
    pub org: bool,
    /// Whether the check requires the role to be assigned to the request's user.
    pub user: bool,
}

impl AuthzEnsure {
    /// Requires a role in realm, scoped to both the request org and user.
    pub fn realm(realm: &str) -> Self {
        Self {
            realm: realm.to_owned(),
            org: true,
            user: true,
        }
    }
    /// Drops the org scoping, for a realm not tied to a single org, e.g. system.
    pub const fn skip_org(mut self) -> Self {
        self.org = false;
        self
    }
    /// Drops the user assignment requirement, e.g. for a public realm.
    pub const fn skip_user(mut self) -> Self {
        self.user = false;
        self
    }
}

#[async_trait]
pub trait AuthzEnsureContext<'a>
where
    Self: AuthzCacheContext<'a>,
{
    /// Requires a role satisfying check, errors with the configured authz_err
    /// when none matches. Consumer apps are expected to wrap this in their own
    /// per realm guards so resolvers only spell out the realm they need, e.g.
    /// #[query(check = org)] over a trait calling authz_ensure here.
    async fn authz_ensure(&self, check: AuthzEnsure) -> Res<()> {
        let v = self.authz_with_cache(check).await?;
        if v.is_none() {
            return Err(self.authz_err().clone());
        }
        Ok(())
    }
}

#[async_trait]
impl<'a> AuthzEnsureContext<'a> for Context<'a> {
}
