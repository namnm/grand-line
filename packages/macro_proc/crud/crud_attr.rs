use crate::prelude::*;

/// Parsed attribute for the create/search/count/detail/update/delete macros.
#[field_names]
pub struct CrudAttr {
    /// When true, the resolver body supplies its own inputs instead of the
    /// generated ones (e.g. id/data for update, filter/order_by/page for search).
    pub resolver_inputs: bool,
    /// When true, the resolver body supplies its own output instead of the
    /// generated model-based one.
    pub resolver_output: bool,
    /// When true (delete only), adds a permanent: Option<bool> input so callers
    /// can request a hard delete instead of a soft delete.
    pub permanent: bool,
    #[field_names(skip)]
    pub model: String,
    #[field_names(skip)]
    pub ra: ResolverTyAttr,
}
impl TryFrom<Attr> for CrudAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        Ok(Self {
            resolver_inputs: a.bool(Self::FIELD_RESOLVER_INPUTS)?.unwrap_or_default(),
            resolver_output: a.bool(Self::FIELD_RESOLVER_OUTPUT)?.unwrap_or_default(),
            permanent: a
                .bool(Self::FIELD_PERMANENT)?
                .unwrap_or(FEATURE_RESOLVER_DELETE_PERMANENT),
            model: a.model_from_first_path()?,
            ra: a.try_into()?,
        })
    }
}
impl AttrValidate for CrudAttr {
    fn attr_fields(a: &Attr) -> Vec<String> {
        Self::FIELDS
            .iter()
            .copied()
            .map(|f| f.to_owned())
            .filter(|f| {
                if a.attr == MacroTy::Delete {
                    true
                } else {
                    f != Self::FIELD_PERMANENT
                }
            })
            .chain(ResolverTyAttr::attr_fields(a))
            .chain(a.first_path.iter().cloned())
            .collect()
    }
}

impl CrudAttr {
    /// Checks that the resolver item's inputs/output shape agrees with the
    /// resolver_inputs/resolver_output flags and that tx/ctx are consistent,
    /// returns an Err describing the mismatch otherwise.
    pub fn validate(&self, r: &ResolverTyItem) -> SynRes<()> {
        let ResolverTyItem {
            gql_name,
            inputs,
            output,
            span,
            ..
        } = &r;
        if !self.resolver_inputs && !inputs.to_string().is_empty() {
            let msg = format!("{gql_name} inputs should be empty unless resolver_inputs=true, found {inputs}");
            return Err(SynErr::new(*span, msg));
        }
        if !self.resolver_output {
            if output.to_string() != "()" {
                let msg = format!("{gql_name} output should be empty unless resolver_output=true, found {output}");
                return Err(SynErr::new(*span, msg));
            }
            if !self.ra.tx || !self.ra.ctx {
                let msg = format!("{gql_name} output requires tx, ctx");
                return Err(SynErr::new(*span, msg));
            }
        }
        if self.resolver_inputs && self.resolver_output {
            let msg = format!(
                "{gql_name} should use #[query] or #[mutation] instead since both resolver_inputs=true and resolver_output=true",
            );
            return Err(SynErr::new(*span, msg));
        }
        if self.ra.tx && !self.ra.ctx {
            let msg = format!("{gql_name} tx requires ctx");
            return Err(SynErr::new(*span, msg));
        }
        Ok(())
    }
}
