use crate::prelude::*;

/// Resolves the org referenced by the request headers, without checking
/// whether the caller is authorized for it.
#[async_trait]
pub trait AuthzHttpContext<'a>
where
    Self: CoreContext<'a> + HttpContext<'a> + AuthzConfigContext<'a>,
{
    /// Return the org from the request headers, cached per request.
    async fn org_unchecked(&self) -> Res<Arc<OrgMinimal>> {
        let arc = self.cache(|| self.org_unchecked_without_cache()).await?;
        Ok(arc)
    }

    /// Look up the org by the configured org id header, erroring if the
    /// header is missing or the org does not exist.
    async fn org_unchecked_without_cache(&self) -> Res<OrgMinimal> {
        let k = self.authz_config().org_id_header_key;
        let v = self.get_header(k)?.trim().to_owned();
        if v.is_empty() {
            return Err(MyErr::HeaderOrgId404.into());
        }

        let org_impl = self.authz_org_impl()?;
        let tx = &*self.tx().await?;

        org_impl
            .find_by_id(&v, tx)
            .await?
            .ok_or_else(|| self.authz_err().clone())
    }
}

#[async_trait]
impl<'a> AuthzHttpContext<'a> for Context<'a> {
}
