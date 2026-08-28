use crate::prelude::*;

/// Parsed attribute shared by #[query]/#[mutation] and the crud macros.
#[field_names]
pub struct ResolverTyAttr {
    /// Whether the generated resolver receives the ctx: &Context<'_> parameter.
    pub ctx: bool,
    /// Whether the generated resolver receives the db: &ConnX<'_> parameter.
    pub db: bool,
    /// Whether to add an include_deleted input controlling soft-deleted rows.
    pub include_deleted: bool,
    /// Whether the resolver queues a subscription event for the row it changed.
    pub publish: bool,
    /// Guards called on ctx in order before the body runs, each is a method
    /// the consumer brings into scope, e.g. check(authenticated, org).
    pub check: Vec<CheckAttr>,
    /// Whether to apply the caller's authz row filter to this resolver's query.
    pub authz_row: bool,
    #[field_names(skip)]
    pub inner: Attr,
}

impl TryFrom<Attr> for ResolverTyAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        Ok(Self {
            ctx: a.bool(Self::FIELD_CTX)?.unwrap_or(FEATURE_RESOLVER_CTX),
            db: a.bool(Self::FIELD_DB)?.unwrap_or(FEATURE_RESOLVER_DB),
            include_deleted: a
                .bool(Self::FIELD_INCLUDE_DELETED)?
                .unwrap_or(FEATURE_RESOLVER_INCLUDE_DELETED),
            publish: a.bool(Self::FIELD_PUBLISH)?.unwrap_or(true),
            check: CheckAttr::parse(&a, Self::FIELD_CHECK)?,
            authz_row: a.bool(Self::FIELD_AUTHZ_ROW)?.unwrap_or(FEATURE_RESOLVER_AUTHZ_ROW),
            inner: a,
        })
    }
}

impl AttrValidate for ResolverTyAttr {
    fn attr_fields(a: &Attr) -> Vec<String> {
        Self::FIELDS
            .iter()
            .copied()
            .map(|f| f.to_owned())
            .filter(|f| {
                if TY_INCLUDE_DELETED.contains(&a.attr) {
                    true
                } else {
                    f != Self::FIELD_INCLUDE_DELETED
                }
            })
            .filter(|f| {
                if TY_PUBLISH.contains(&a.attr) {
                    true
                } else {
                    f != Self::FIELD_PUBLISH
                }
            })
            .collect()
    }
}
