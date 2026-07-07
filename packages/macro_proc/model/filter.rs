use crate::prelude::*;

/// Add every applicable comparison filter field (eq, ne, is_in, gt/gte/lt/lte,
/// like/ilike, starts_with/ends_with, ...) for one model field, which operators
/// apply depends on the field's unwrapped type, bool and id/_id fields get fewer.
pub fn filter(f: &Field, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>) -> SynRes<()> {
    push(f, struk, query, "eq")?;
    push(f, struk, query, "ne")?;
    let (_, uw_str) = unwrap_option_str(f.ty.to_token_stream());
    if uw_str == "bool" {
        return Ok(());
    }
    push(f, struk, query, "is_in")?;
    push(f, struk, query, "is_not_in")?;
    let name_str = f.ident.to_token_stream().to_string();
    if name_str == "id" || name_str.ends_with("_id") {
        return Ok(());
    }
    push(f, struk, query, "gt")?;
    push(f, struk, query, "gte")?;
    push(f, struk, query, "lt")?;
    push(f, struk, query, "lte")?;
    if uw_str != "String" {
        return Ok(());
    }
    if cfg!(not(feature = "postgres")) {
        push(f, struk, query, "like")?;
        push(f, struk, query, "not_like")?;
    }
    if cfg!(feature = "postgres") {
        push(f, struk, query, "ilike")?;
        push(f, struk, query, "not_ilike")?;
    }
    push(f, struk, query, "starts_with")?;
    push(f, struk, query, "ends_with")?;
    Ok(())
}

fn push(f: &Field, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>, op_str: &str) -> SynRes<()> {
    let col = f.ident.to_token_stream().to_string().to_pascal_case().ts2_or_err()?;
    let op = op_str.ts2_or_err()?;
    let mut gql_op = op_str.to_owned();
    // unwrap Option<type>
    // the type can be generic such as Box<type>
    let (opt, mut ty) = unwrap_option(f.ty.to_token_stream())?;
    // handle special operators
    if op_str == "is_in" || op_str == "is_not_in" {
        gql_op = op_str.replace("is_", "");
        ty = quote!(Vec<#ty>);
    }
    // struct struct_field_some_op
    // graphql structField_someOp
    let pg = hashmap! {
        "ilike" => "iLike",
        "not_ilike" => "notILike",
    };
    let mut name = f.ident.to_token_stream();
    let mut gql_name = name.to_string().to_lower_camel_case();
    if op_str != "eq" {
        name = format!("{name}_{gql_op}").to_snake_case().ts2_or_err()?;
        let gql_op_camel = pg
            .get(op_str)
            .copied()
            .map(|f| f.to_owned())
            .unwrap_or_else(|| gql_op.to_lower_camel_case());
        gql_name = format!("{gql_name}_{gql_op_camel}");
    }
    // push struk
    let opt_eq_ne = opt && (op_str == "eq" || op_str == "ne");
    ty = if opt_eq_ne {
        quote!(Undefined<#ty>)
    } else {
        quote!(Option<#ty>)
    };
    struk.push(quote! {
        #[graphql(name = #gql_name)]
        pub #name: #ty,
    });
    // push query
    let q = if opt_eq_ne {
        let op_null = if op_str == "eq" {
            quote!(is_null)
        } else {
            quote!(is_not_null)
        };
        quote! {
            if let Undefined::Null = f.#name {
                c = c.add(Column::#col.#op_null());
            } else if let Undefined::Value(v) = f.#name {
                c = c.add(Column::#col.#op(v));
            }
        }
    } else if pg.contains_key(op_str) {
        quote! {
            if let Some(v) = f.#name {
                use sea_query::extension::postgres::PgExpr;
                c = c.add(Expr::col(Column::#col).#op(v));
            }
        }
    } else {
        quote! {
            if let Some(v) = f.#name {
                c = c.add(Column::#col.#op(v));
            }
        }
    };
    query.push(q);
    Ok(())
}
