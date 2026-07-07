use crate::prelude::*;

/// Parsed arguments of #[resolver(fn_path, sql_dep = "...", ...)] on a model
/// field, f is the function to call (defaults to resolve_<field>), sql_dep lists
/// the entity columns the resolver depends on for GQL_SELECT.
#[field_names]
pub struct ResolverAttr {
    #[field_names(skip)]
    pub f: Ident,
    pub sql_dep: Vec<String>,
    #[field_names(skip)]
    pub ra: ResolverTyAttr,
    #[field_names(skip)]
    pub inner: Attr,
}
impl TryFrom<Attr> for ResolverAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        let f = if let Some(custom) = &a.first_path {
            format_ident!("{custom}")
        } else {
            let field = a.field_name()?;
            format_ident!("resolve_{field}")
        };
        let sql_dep = a
            .str(Self::FIELD_SQL_DEP)?
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_owned())
            .collect();
        let ra = a.clone().try_into()?;
        Ok(Self {
            f,
            sql_dep,
            ra,
            inner: a,
        })
    }
}
impl AttrValidate for ResolverAttr {
    fn attr_fields(a: &Attr) -> Vec<String> {
        Self::FIELDS
            .iter()
            .copied()
            .map(|f| f.to_owned())
            .chain(ResolverTyAttr::attr_fields(a))
            .chain(a.first_path.iter().cloned())
            .collect()
    }
}
