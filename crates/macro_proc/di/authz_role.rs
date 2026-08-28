use crate::prelude::*;

/// Parsed authz_role(...) attribute.
#[field_names]
pub struct AuthzRoleAttr {
    /// Realm granted across every org, tried when the requested realm has no
    /// matching role, e.g. fallback = "system".
    pub fallback: Option<String>,
}
impl TryFrom<Attr> for AuthzRoleAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        Ok(Self {
            fallback: a.str(Self::FIELD_FALLBACK)?,
        })
    }
}
impl AttrValidate for AuthzRoleAttr {
    fn attr_fields(_attr: &Attr) -> Vec<String> {
        Self::FIELDS.iter().copied().map(|f| f.to_owned()).collect()
    }
}

/// Proc-macro entry for #[authz_role], implements AuthzRoleModel on the entity
/// so it can back the framework's default AuthzRoleImpl.
pub fn gen_authz_role(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttrParse);
    try_gen_authz_role(attr, item).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_authz_role(attr: AttrParse, item: TokenStream) -> SynRes<TokenStream> {
    let a = attr.into_inner::<AuthzRoleAttr>("authz_role")?;
    let m = DiModel::parse("authz_role", item)?;
    m.require(&["id", "realm", "col_policy", "row_policy", "org_id"])?;

    let fallback = if let Some(f) = a.fallback {
        quote!(Some(#f))
    } else {
        quote!(None)
    };
    let org_id = gen_impl_org_id();

    let impls = quote! {
        #org_id
        impl AuthzRoleModel for Entity {
            const FALLBACK_REALM: Option<&'static str> = #fallback;

            fn col_realm() -> Self::C {
                Column::Realm
            }
            fn authz_role_match(m: Self::M) -> Res<AuthzRoleMatch> {
                Ok(AuthzRoleMatch {
                    role_id: m.id,
                    col_policy: ColPolicy::from_json(m.col_policy)?,
                    row_policy: RowPolicy::from_json(m.row_policy)?,
                })
            }
        }
    };
    Ok(m.g(&impls))
}
