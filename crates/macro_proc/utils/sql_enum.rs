use crate::prelude::*;

/// Entry point for the #[sql_enum] attribute macro, attaches the derives
/// needed for a db-backed String enum (gql_enum, EnumIter, DeriveActiveEnum,
/// stored as a snake_case String column).
pub fn gen_sql_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = Into::<Ts2>::into(attr);
    let item = Into::<Ts2>::into(item);

    quote! {
        #[gql_enum]
        #[derive(EnumIter, DeriveActiveEnum)]
        #[sea_orm(
            rs_type = "String",
            db_type = "String(StringLen::N(255))",
            rename_all = "snake_case",
        )]
        #attr
        #item
    }
    .into()
}
