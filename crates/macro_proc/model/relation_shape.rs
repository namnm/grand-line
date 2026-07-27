use crate::prelude::*;

// ---------------------------------------------------------------------------
// Relation shape resolution
// ---------------------------------------------------------------------------

/// Column mapping for a belongs_to / has_one / has_many / many_to_many relation.
/// self_col is the column on the owning entity, to_col is the column on the target
/// entity, such that self.self_col and target.to_col correlate one relation instance.
/// For many_to_many, both sides are always Id, the join table is handled separately
/// by many_to_many_reachable_ids / many_to_many_filtered_self_ids.
pub struct RelationShape {
    pub self_col: Ts2,
    pub to_col: Ts2,
}

/// Compute the self_col / to_col pair for a relation, based on its RelationTy,
/// see RelationShape for what the two columns mean.
pub fn relation_shape(ty: &RelationTy, a: &RelationAttr) -> SynRes<RelationShape> {
    match ty {
        RelationTy::BelongsTo => {
            let self_col = a.key_str()?.to_pascal_case().ts2_or_err()?;
            Ok(RelationShape {
                self_col,
                to_col: quote!(Id),
            })
        }
        RelationTy::HasOne | RelationTy::HasMany => {
            let to_col = a.key_str()?.to_pascal_case().ts2_or_err()?;
            Ok(RelationShape {
                self_col: quote!(Id),
                to_col,
            })
        }
        RelationTy::ManyToMany => Ok(RelationShape {
            self_col: quote!(Id),
            to_col: quote!(Id),
        }),
    }
}

// ---------------------------------------------------------------------------
// Condition and subquery builders
// ---------------------------------------------------------------------------

/// Build a Condition::all() matching column::col against the owning row's id,
/// bound as id in the caller's scope, used by has_many list and count resolvers.
pub fn has_many_condition(column: &Ts2, col: &Ts2) -> Ts2 {
    quote! {
        Condition::all().add(#column::#col.eq(id))
    }
}

/// many_to_many: given the owning row's id (bound as id in the caller's scope),
/// build Condition::all().add(target.id IN (target ids reachable through the join table)).
/// Shared by the relation's own list resolver and its _count resolver, both of which
/// scope the target query down to one owning row.
pub fn many_to_many_condition(a: &RelationAttr) -> SynRes<Ts2> {
    let to = a.to()?;
    let to_column = ty_column(&to)?;
    let through = a.through()?;
    let through_column = ty_column(&through)?;
    let key_col = a.key_str()?.to_pascal_case().ts2_or_err()?;
    let other_key_col = a.other_key()?.to_string().to_pascal_case().ts2_or_err()?;
    let include_deleted = get_include_deleted(a.include_deleted);

    let r = quote! {{
        let sub = #through::find()
            .select_only()
            .column(#through_column::#other_key_col)
            .filter(#through_column::#key_col.eq(id))
            .include_deleted(#include_deleted.unwrap_or_default())
            .into_query();
        Condition::all().add(#to_column::Id.in_subquery(sub))
    }};
    Ok(r)
}

/// many_to_many: the reverse direction of many_to_many_reachable_ids - given a condition
/// on the target entity (cond, typically a nested filter's into_condition() or its
/// negation), build a SelectStatement of owning-row ids that have at least one related
/// target row matching cond. Used by the relation's _some/_none/_every filter fields.
pub fn many_to_many_filter(a: &RelationAttr, cond: &Ts2) -> SynRes<Ts2> {
    let to = a.to()?;
    let to_column = ty_column(&to)?;
    let through = a.through()?;
    let through_column = ty_column(&through)?;
    let key_col = a.key_str()?.to_pascal_case().ts2_or_err()?;
    let other_key_col = a.other_key()?.to_string().to_pascal_case().ts2_or_err()?;
    let include_deleted = if a.include_deleted {
        quote!(f.has_deleted_at())
    } else {
        quote!(false)
    };

    let r = quote! {{
        let sub = #to::find()
            .select_only()
            .column(#to_column::Id)
            .include_deleted(#include_deleted)
            .filter(#cond)
            .into_query();
        #through::find()
            .select_only()
            .column(#through_column::#key_col)
            // through id should be composite-like behavior here
            .include_deleted(false)
            .filter(#through_column::#other_key_col.in_subquery(sub))
            .into_query()
    }};
    Ok(r)
}

/// belongs_to / has_one / has_many: build the subquery selecting the target column
/// matching the nested condition cond, paired with the self column to test it against.
pub fn relation_shape_filter(r: &GenRelation, cond: &Ts2) -> SynRes<Ts2> {
    let shape = relation_shape(&r.ty, &r.a)?;
    let to = r.a.to()?;
    let to_column = ty_column(&to)?;
    let to_col = shape.to_col;
    let include_deleted = if r.a.include_deleted {
        quote!(f.has_deleted_at())
    } else {
        quote!(false)
    };

    let r = quote! {
        #to::find()
            .select_only()
            .column(#to_column::#to_col)
            .include_deleted(#include_deleted)
            .filter(#cond)
            .into_query()
    };
    Ok(r)
}
