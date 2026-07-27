use crate::prelude::*;

/// Parsed attribute shared by #[query]/#[mutation] and the crud macros.
#[field_names]
pub struct ResolverTyAttr {
    /// Whether the generated resolver opens a db transaction.
    pub tx: bool,
    /// Whether the generated resolver receives the ctx: &Context<'_> parameter.
    pub ctx: bool,
    /// Whether to add an include_deleted input controlling soft-deleted rows.
    pub include_deleted: bool,
    pub auth: Option<AuthAttr>,
    pub authz: Option<AuthzAttr>,
    /// Whether to apply the caller's authz row filter to this resolver's query.
    pub authz_row: bool,
    #[field_names(skip)]
    pub inner: Attr,
}

impl ResolverTyAttr {
    /// True when either auth is set to a check other than unauthenticated, or
    /// authz is set, meaning the resolver requires an authenticated caller.
    pub fn has_auth(&self) -> bool {
        let auth = self.auth.as_ref();
        let auth = auth.is_some() && !auth.is_some_and(|v| v.unauthenticated);
        let authz = self.authz.as_ref();
        let authz = authz.is_some();
        auth || authz
    }
}

impl TryFrom<Attr> for ResolverTyAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        Ok(Self {
            tx: a.bool(Self::FIELD_TX)?.unwrap_or(FEATURE_RESOLVER_TX),
            ctx: a.bool(Self::FIELD_CTX)?.unwrap_or(FEATURE_RESOLVER_CTX),
            include_deleted: a
                .bool(Self::FIELD_INCLUDE_DELETED)?
                .unwrap_or(FEATURE_RESOLVER_INCLUDE_DELETED),
            auth: a.nested_with_path_into::<AuthAttr>(Self::FIELD_AUTH)?.map(|(_, a)| a),
            authz: a.nested_into::<AuthzAttr>(Self::FIELD_AUTHZ)?,
            authz_row: a.bool(Self::FIELD_AUTHZ_ROW)?.unwrap_or(FEATURE_RESOLVER_AUTHZ_ROW),
            inner: a,
        })
    }
}

impl AttrValidate for ResolverTyAttr {
    fn attr_fields(a: &Attr) -> Vec<String> {
        let f = Self::FIELDS.iter().copied().map(|f| f.to_owned()).filter(|f| {
            if TY_INCLUDE_DELETED.contains(&a.attr) {
                true
            } else {
                f != Self::FIELD_INCLUDE_DELETED
            }
        });
        #[cfg(not(feature = "auth"))]
        let f = f.filter(|f| f != Self::FIELD_AUTH);
        #[cfg(not(feature = "authz"))]
        let f = f.filter(|f| f != Self::FIELD_AUTHZ);
        f.collect()
    }
}
