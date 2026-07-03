use crate::prelude::*;

pub fn filter_and_or_not(f: &Ts2, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>) -> SynRes<()> {
    push_and_or(f, struk, query, "and")?;
    push_and_or(f, struk, query, "or")?;
    push_not(f, struk, query);
    Ok(())
}

fn push_and_or(f: &Ts2, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>, op_str: &str) -> SynRes<()> {
    let op = op_str.ts2_or_err()?;
    let gql_op = op_str.to_uppercase();
    let cond = if op_str == "and" {
        quote!(all)
    } else {
        quote!(any)
    };

    struk.push(quote! {
        #[graphql(name = #gql_op)]
        pub #op: Option<Vec<#f>>,
    });
    query.push(quote! {
        if let Some(v) = f.#op {
            let mut #op = Condition::#cond();
            for f in v {
                #op = #op.add(f.into_condition());
            }
            c = c.add(#op);
        }
    });
    Ok(())
}

fn push_not(f: &Ts2, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>) {
    struk.push(quote! {
        #[graphql(name = "NOT")]
        pub not: Option<Box<#f>>,
    });
    query.push(quote! {
        if let Some(f) = f.not {
            let f = *f;
            c = c.add(Condition::not(f.into_condition()));
        }
    });
}
