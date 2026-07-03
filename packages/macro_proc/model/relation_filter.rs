use crate::prelude::*;

/// Add <field>_some / <field>_none / <field>_every filter fields for a relationship.
/// _some matches rows where at least one related row satisfies the nested filter.
/// _none matches rows where no related row satisfies the nested filter.
/// _every matches rows where every related row satisfies the nested filter
/// (vacuously true when there is no related row).
pub fn relation_filter(r: &GenRelation, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>) -> SynRes<()> {
    push(r, struk, query, "some")?;
    push(r, struk, query, "none")?;
    push(r, struk, query, "every")?;
    Ok(())
}

fn push(r: &GenRelation, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>, op_str: &str) -> SynRes<()> {
    let base = r.a.name()?.to_string();
    let name = format!("{base}_{op_str}").ts2_or_err()?;
    let gql_base = base.to_lower_camel_case();
    let gql_name = format!("{gql_base}_{op_str}");

    let to = r.a.to()?;
    let filter = ty_filter(&to)?;
    // _every is _none of the negated nested filter: no related row fails to match.
    let negate = op_str == "every";
    let (self_col, sub) = self_col_and_subquery(r, negate)?;
    let in_fn = if op_str == "some" {
        quote!(in_subquery)
    } else {
        quote!(not_in_subquery)
    };

    struk.push(quote! {
        #[graphql(name = #gql_name)]
        pub #name: Option<Box<#filter>>,
    });
    query.push(quote! {
        if let Some(f) = f.#name {
            let f = *f;
            let sub = #sub;
            c = c.add(Column::#self_col.#in_fn(sub));
        }
    });
    Ok(())
}

/// Compute the column on the owning entity to test with in_subquery/not_in_subquery,
/// and the subquery expression selecting the matching side of that column,
/// filtered down by the nested filter f (or its negation, when negate is set).
fn self_col_and_subquery(r: &GenRelation, negate: bool) -> SynRes<(Ts2, Ts2)> {
    let mut cond = quote!(f.into_condition());
    if negate {
        cond = quote!(Condition::not(#cond));
    }

    if r.ty == RelationTy::ManyToMany {
        let sub = many_to_many_filter(&r.a, &cond)?;
        return Ok((quote!(Id), sub));
    }

    let shape = relation_shape(&r.ty, &r.a)?;
    let sub = relation_shape_filter(r, &cond)?;
    Ok((shape.self_col, sub))
}
