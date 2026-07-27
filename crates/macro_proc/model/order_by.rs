use crate::prelude::*;

/// Add the Asc and Desc order_by variants for one model field.
pub fn order_by(f: &Field, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>) -> SynRes<()> {
    push(f, struk, query, "asc")?;
    push(f, struk, query, "desc")?;
    Ok(())
}
fn push(f: &Field, struk: &mut Vec<Ts2>, query: &mut Vec<Ts2>, direction_str: &str) -> SynRes<()> {
    // sea_orm generated order_by_#direction(Column::Name)
    let column = f.ident.to_token_stream().to_string().to_pascal_case().ts2_or_err()?;
    let direction_fn = format!("order_by_{direction_str}").to_snake_case().ts2_or_err()?;
    // enum EnumField
    // graphql EnumField
    let gql_name = format!("{column}_{direction_str}").to_pascal_case();
    let name = gql_name.ts2_or_err()?;
    // push
    struk.push(quote! {
        #[graphql(name = #gql_name)]
        #name,
    });
    query.push(quote! {
        Self::#name => q.#direction_fn(Column::#column),
    });
    Ok(())
}
