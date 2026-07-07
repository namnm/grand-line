mod crud;
mod model;
mod resolver_ty;
mod utils;

#[allow(ambiguous_glob_reexports, dead_code, unused_imports)]
mod prelude {
    pub use crate::crud::*;
    pub use crate::model::*;
    pub use crate::resolver_ty::*;
    pub use crate::utils::*;
    pub use _utils::*;
    pub use _utils_proc::*;
    pub use proc_macro::TokenStream;
    use_common_macro_utils!();
    use_common_std!();
}
use crate::prelude::*;

// ============================================================================
// model

/// Derives the full CRUD type family (sea_orm entity, GraphQL type,
/// ActiveModel, Filter, OrderBy) for a model struct.
#[proc_macro_attribute]
pub fn model(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_model(attr, item)
}

/// Turns a fn into a relation/field resolver returning Res<Search<..>>, parent,
/// ctx, and tx are auto injected, the fn should take no declared inputs.
#[proc_macro_attribute]
pub fn many_resolver(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_many_resolver(attr, item)
}

/// Turns a fn into a relation/field resolver returning Res<Count>, parent,
/// ctx, and tx are auto injected, the fn should take no declared inputs.
#[proc_macro_attribute]
pub fn count_resolver(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_count_resolver(attr, item)
}

/// Turns a fn into a plain field resolver, parent, ctx, and tx are auto
/// injected, the fn should take no declared inputs.
#[proc_macro_attribute]
pub fn field_resolver(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_field_resolver(attr, item)
}

/// Turns a fn into a relation/field resolver returning Res<Detail>, parent,
/// ctx, and tx are auto injected, the fn should take no declared inputs.
#[proc_macro_attribute]
pub fn one_resolver(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_one_resolver(attr, item)
}

// ============================================================================
// resolver

/// Registers the annotated fn as a root Query field, inputs/output/body are
/// used as written.
#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_query(attr, item)
}

/// Registers the annotated fn as a root Mutation field, inputs/output/body
/// are used as written.
#[proc_macro_attribute]
pub fn mutation(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_mutation(attr, item)
}

// ============================================================================
// crud

/// Registers a create mutation for a model, default inputs are
/// data: <Model>Create and the output is the model's Gql type.
#[proc_macro_attribute]
pub fn create(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_create(attr, item)
}

/// Registers a list query for a model, default inputs are
/// filter/order_by/page and the output is Vec<Gql>.
#[proc_macro_attribute]
pub fn search(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_search(attr, item)
}

/// Registers a count query for a model, default input is filter and the
/// output is u64.
#[proc_macro_attribute]
pub fn count(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_count(attr, item)
}

/// Registers a single-row query for a model, default input is id and the
/// output is Option<Gql>.
#[proc_macro_attribute]
pub fn detail(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_detail(attr, item)
}

/// Registers an update mutation for a model, default inputs are
/// id/data: <Model>Update and the output is the model's Gql type.
#[proc_macro_attribute]
pub fn update(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_update(attr, item)
}

/// Registers a delete mutation for a model, default input is id (plus
/// permanent when enabled) and the output is the model's Gql type.
#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_delete(attr, item)
}

// ============================================================================
// utils

/// Attaches the derives needed for a db-backed String enum (gql_enum,
/// EnumIter, DeriveActiveEnum, stored as a snake_case String column).
#[proc_macro_attribute]
pub fn sql_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_sql_enum(attr, item)
}

/// Attaches the derives needed for a GraphQL-facing Copy enum.
#[proc_macro_attribute]
pub fn gql_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_gql_enum(attr, item)
}

/// Attaches the derives needed for a GraphQL input struct.
#[proc_macro_attribute]
pub fn gql_input(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_gql_input(attr, item)
}

/// Helper to quickly create a filter with concise syntax.
#[proc_macro]
pub fn filter(item: TokenStream) -> TokenStream {
    expr_struct(item, "Filter", "Some", "")
}

/// Helper to quickly create an order_by with concise syntax.
#[proc_macro]
pub fn order_by(item: TokenStream) -> TokenStream {
    gen_order_by(item)
}

/// Helper to quickly create an AmWrapper<AmCreate, E, A> with concise syntax
/// and convert all string literals into String automatically.
/// Call .exec_without_ctx(db) or .exec(ctx) to execute.
#[proc_macro]
pub fn am_create(item: TokenStream) -> TokenStream {
    expr_struct_am_wrapper(item, "ActiveModel", "AmCreate")
}

/// Helper to quickly create an AmWrapper<AmUpdate, E, A> with concise syntax
/// and convert all string literals into String automatically.
/// Call .exec_without_ctx(db) or .exec(ctx) to execute.
#[proc_macro]
pub fn am_update(item: TokenStream) -> TokenStream {
    expr_struct_am_wrapper(item, "ActiveModel", "AmUpdate")
}

/// Helper to quickly create an AmWrapper<AmSoftDelete, E, A> with concise syntax
/// and convert all string literals into String automatically.
/// Call .exec_without_ctx(db) or .exec(ctx) to execute.
#[proc_macro]
pub fn am_soft_delete(item: TokenStream) -> TokenStream {
    expr_struct_am_wrapper(item, "ActiveModel", "AmSoftDelete")
}

/// Helper to quickly create an AmCreateMany from a model name and an array of
/// field-only blocks, e.g. am_create_many!(Todo, [{ content: "a" }, { content: "b" }]).
/// Call .exec_without_ctx(db) or .exec(ctx) to execute, same as am_create!.
/// By default the returned models are reconstructed in memory, no round trip
/// after the bulk INSERT. Chain .returning() when the row needs to reflect
/// db-side defaults or triggers (custom db) the client can't see, this uses
/// RETURNING on postgres and an extra SELECT by id on other backends.
#[proc_macro]
pub fn am_create_many(item: TokenStream) -> TokenStream {
    expr_array_am_wrapper(item, "ActiveModel", "AmCreate")
}

/// Helper to quickly create an AmUpdateMany from a model name and an array of
/// field-only blocks, e.g. am_update_many!(Todo, [{ id: "1", done: true }]).
/// Call .exec_without_ctx(db) or .exec(ctx) to execute, same as am_update!.
/// Each item can carry a different id and fields, sea_orm has no single-statement
/// bulk update for that, so this runs one UPDATE per row under the hood.
#[proc_macro]
pub fn am_update_many(item: TokenStream) -> TokenStream {
    expr_array_am_wrapper(item, "ActiveModel", "AmUpdate")
}

/// Helper to quickly create an AmSoftDeleteMany from a model name and an array of
/// field-only blocks, e.g. am_soft_delete_many!(Todo, [{ id: "1" }, { id: "2" }]).
/// Call .exec_without_ctx(db) or .exec(ctx) to execute, same as am_soft_delete!.
/// Runs one UPDATE per row under the hood, same reason as am_update_many!.
#[proc_macro]
pub fn am_soft_delete_many(item: TokenStream) -> TokenStream {
    expr_array_am_wrapper(item, "ActiveModel", "AmSoftDelete")
}

/// Automatically derive ThisErr, GrandLineErrDerive, Debug.
#[proc_macro_attribute]
pub fn grand_line_err(attr: TokenStream, item: TokenStream) -> TokenStream {
    gen_grand_line_err(attr, item)
}

/// Automatically implement GrandLineErrImpl to handle error better.
#[proc_macro_derive(GrandLineErrDerive, attributes(client, code))]
pub fn grand_line_err_derive(item: TokenStream) -> TokenStream {
    gen_grand_line_err_derive(item)
}
