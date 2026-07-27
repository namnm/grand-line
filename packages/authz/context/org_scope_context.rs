use crate::prelude::*;

/// Generic CRUD scoping for AuthzImplOrgId entities, covers the "scope by
/// the current authz org" boilerplate shared by every consumer models.
#[async_trait]
pub trait AuthzOrgScopeContext<'a>
where
    Self: AuthzContext<'a>,
{
    /// Search scoped to the current authz org.
    async fn authz_org_search<E>(&self) -> Res<Search<E::O>>
    where
        E: AuthzImplOrgId,
    {
        let org_id = self.authz().await?;
        Ok(Search::default().add(E::col_org_id().eq(org_id)))
    }

    /// Count/detail filter scoped to the current authz org.
    async fn authz_org_filter<E>(&self) -> Res<Filter>
    where
        E: AuthzImplOrgId,
    {
        let org_id = self.authz().await?;
        Ok(Filter::default().add(E::col_org_id().eq(org_id)))
    }

    /// Fetches a row by id scoped to the current authz org.
    async fn authz_org_one_or_404<E>(&self, id: &str) -> Res<E::M>
    where
        E: AuthzImplOrgId,
    {
        let org_id = self.authz().await?;
        let tx = &*self.tx().await?;
        E::find()
            .filter_by_id(id)
            .filter(E::col_org_id().eq(org_id))
            .one_or_404(tx)
            .await
    }

    /// Soft-deletes a row scoped to the current authz org.
    async fn authz_org_soft_delete<E>(&self, id: &str) -> Res<E::G>
    where
        E: AuthzImplOrgId,
    {
        let org_id = self.authz().await?;
        let tx = &*self.tx().await?;
        E::find()
            .filter_by_id(id)
            .filter(E::col_org_id().eq(&org_id))
            .exists_or_404(tx)
            .await?;
        E::soft_delete_by_id(id)?
            .filter(E::col_org_id().eq(&org_id))
            .exec(tx)
            .await?;
        Ok(E::G::from_id(id))
    }
}

#[async_trait]
impl<'a> AuthzOrgScopeContext<'a> for Context<'a> {
}
