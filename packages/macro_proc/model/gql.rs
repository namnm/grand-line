use crate::prelude::*;

// ============================================================================
// Field attribute helpers

/// Extract doc-comment strings from a field's attributes.
/// Each `///` line becomes one String entry (with the leading space preserved).
pub fn attr_doc_strs(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if !attr.path().is_ident("doc") {
                return None;
            }
            if let Meta::NameValue(nv) = &attr.meta
                && let Expr::Lit(el) = &nv.value
                && let Lit::Str(s) = &el.lit
            {
                return Some(s.value());
            }
            None
        })
        .collect()
}

/// Extract the graphql `name` override and all remaining graphql args from a
/// field's attributes.
///
/// Returns `(name_override, extra_args)` where `extra_args` is a `Ts2` ready
/// to be spliced into `#[graphql(name = ..., #extra_args)]`.
/// The `name` key is consumed and returned separately so the caller controls
/// which effective name ends up in the attribute.
pub fn attr_graphql_info(attrs: &[Attribute]) -> (Option<String>, Ts2) {
    let mut name_override = None;
    let mut extras: Vec<Ts2> = vec![];

    for attr in attrs {
        if !attr.path().is_ident("graphql") {
            continue;
        }
        let Ok(args) = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) else {
            continue;
        };
        for meta in args {
            match &meta {
                Meta::NameValue(nv) if nv.path.is_ident("name") => {
                    if let Expr::Lit(el) = &nv.value
                        && let Lit::Str(s) = &el.lit
                    {
                        name_override = Some(s.value());
                    }
                }
                _ => extras.push(quote!(#meta,)),
            }
        }
    }

    (name_override, quote!(#(#extras)*))
}

// ============================================================================

pub struct GqlAttr {
    pub struk: Vec<Ts2>,
    pub struk_fields: Vec<String>,
    pub defaults: Vec<Ts2>,
    pub resolver: Vec<Ts2>,
    pub into: Vec<Ts2>,
    pub cols: Vec<Ts2>,
    pub select: Vec<Ts2>,
    pub get_string: Vec<Ts2>,
}

pub fn gql_attr(gql_fields: &[(Field, Vec<Attr>)]) -> SynRes<GqlAttr> {
    let (mut struk, mut struk_fields, mut defaults, mut resolver, mut into, mut cols, mut select, mut get_string) =
        (vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![]);

    for (f, a) in gql_fields {
        let name = f.ident.to_token_stream();
        let ty = f.ty.to_token_stream();
        let (opt, uw_str) = unwrap_option_str(&ty);

        // f.attrs contains only doc and #[graphql(...)] after attr_gql filtering.
        let doc_strs = attr_doc_strs(&f.attrs);
        let (gql_name_override, extra_graphql) = attr_graphql_info(&f.attrs);

        push_struk_resolver(
            &name,
            &ty,
            &mut struk,
            &mut struk_fields,
            &mut resolver,
            attr_is_gql_skip(a),
            gql_name_override.as_deref(),
            &extra_graphql,
            &doc_strs,
        )?;
        push_default(&mut defaults, &name);

        let name_str = name.to_string();
        let col = name.to_string().to_pascal_case().ts2_or_err()?;
        cols.push(quote! {
            m.insert(#name_str, Column::#col);
        });

        // Use the effective graphql name (user override or auto camelCase) so
        // GQL_SELECT maps the correct GraphQL field name to its SQL columns.
        let auto_gql_name = name.to_string().to_lower_camel_case();
        let effective_gql_name: &str = gql_name_override.as_deref().unwrap_or(&auto_gql_name);
        select.push(quote! {
            m.entry(#effective_gql_name).or_insert_with(HashSet::new).insert(#name_str);
        });

        into.push(if opt {
            quote! {
                #name: self.#name,
            }
        } else {
            quote! {
                #name: Some(self.#name),
            }
        });

        if uw_str == "String" {
            get_string.push(quote! {
                Column::#col => self.#name.clone(),
            });
        }
    }

    Ok(GqlAttr {
        struk,
        struk_fields,
        defaults,
        resolver,
        into,
        cols,
        select,
        get_string,
    })
}

pub struct GqlAttrVirtuals {
    pub select: Vec<Ts2>,
}

pub fn gql_attr_virtuals(virtual_resolvers: &[Box<dyn VirtualResolverFn>]) -> SynRes<GqlAttrVirtuals> {
    let mut select = vec![];
    for v in virtual_resolvers {
        let gql_name = v.gql_name()?;
        for name_str in v.sql_dep()? {
            select.push(quote! {
                m.entry(#gql_name).or_insert_with(HashSet::new).insert(#name_str);
            });
        }
    }
    Ok(GqlAttrVirtuals {
        select,
    })
}

pub struct GqlAttrExprs {
    pub struk: Vec<Ts2>,
    pub struk_fields: Vec<String>,
    pub defaults: Vec<Ts2>,
    pub resolver: Vec<Ts2>,
    pub select: Vec<Ts2>,
    pub exprs: Vec<Ts2>,
}

pub fn gql_exprs_ts2(exprs: &[(Field, Vec<Attr>)]) -> SynRes<GqlAttrExprs> {
    let (mut struk, mut struk_fields, mut defaults, mut resolver, mut select, mut gql_exprs) =
        (vec![], vec![], vec![], vec![], vec![], vec![]);

    for (f, e) in exprs {
        let a = e.iter().find(|a| a.attr == VirtualTy::SqlExpr).ok_or_else(|| {
            let span = e.first().map(|a| a.span).unwrap_or_else(Span::call_site);
            let msg = "cannot find VirtualTy::SqlExpr to build select as";
            SynErr::new(span, msg)
        })?;
        let name_str = a.field_name()?;
        let name = name_str.ts2_or_err()?;
        let ty = a.field_ty()?.ts2_or_err()?;

        // f.attrs has the original field attrs (not filtered by attr_gql).
        let doc_strs = attr_doc_strs(&f.attrs);
        let (gql_name_override, extra_graphql) = attr_graphql_info(&f.attrs);

        push_struk_resolver(
            &name,
            &ty,
            &mut struk,
            &mut struk_fields,
            &mut resolver,
            false,
            gql_name_override.as_deref(),
            &extra_graphql,
            &doc_strs,
        )?;
        push_default(&mut defaults, &name);

        let auto_gql_name = name.to_string().to_lower_camel_case();
        let effective_gql_name: &str = gql_name_override.as_deref().unwrap_or(&auto_gql_name);
        select.push(quote! {
            m.entry(#effective_gql_name).or_insert_with(HashSet::new).insert(#name_str);
        });
        let sql_expr = a.raw()?.ts2_or_err()?;
        gql_exprs.push(quote! {
            m.insert(#name_str, #sql_expr);
        });
    }

    Ok(GqlAttrExprs {
        struk,
        struk_fields,
        resolver,
        select,
        exprs: gql_exprs,
        defaults,
    })
}

fn push_struk_resolver(
    name: &Ts2,
    ty: &Ts2,
    struk: &mut Vec<Ts2>,
    struk_fields: &mut Vec<String>,
    resolver: &mut Vec<Ts2>,
    skip_resolver: bool,
    gql_name_override: Option<&str>,
    extra_graphql: &Ts2,
    doc_strs: &[String],
) -> SynRes<()> {
    let (opt, uw) = unwrap_option(ty)?;

    struk.push(quote! {
        pub #name: Option<#uw>,
    });
    struk_fields.push(name.to_string());

    if skip_resolver {
        return Ok(());
    }

    let auto_gql_name = name.to_string().to_lower_camel_case();
    let gql_name: &str = gql_name_override.unwrap_or(&auto_gql_name);
    let unwrap = if opt {
        quote!()
    } else {
        quote! {
            .ok_or(CoreDbErr::GqlResolverNone)?
        }
    };
    let graphql_attr = if extra_graphql.is_empty() {
        quote!(#[graphql(name = #gql_name)])
    } else {
        quote!(#[graphql(name = #gql_name, #extra_graphql)])
    };

    resolver.push(quote! {
        #(#[doc = #doc_strs])*
        #graphql_attr
        pub async fn #name(&self) -> Res<#ty> {
            let v = self.#name.clone()#unwrap;
            Ok(v)
        }
    });
    Ok(())
}

fn push_default(defaults: &mut Vec<Ts2>, name: &Ts2) {
    defaults.push(quote! {
        #name: None,
    });
}
