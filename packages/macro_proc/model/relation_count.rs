use crate::prelude::*;

/// Optional `<field>_count` resolver for `has_many` / `many_to_many` relations,
/// enabled via `count` on the relation attribute (e.g. `#[has_many(count)]`).
/// Mirrors the `count` crud macro, scoped down to this relation via the same
/// extra condition used by the relation's own list resolver.
pub struct GenRelationCount {
    pub ty: RelationTy,
    pub a: RelationAttr,
}

impl GenRelationCount {
    fn extra_cond(&self) -> SynRes<Ts2> {
        match self.ty {
            RelationTy::HasMany => {
                let to = self.a.to()?;
                let column = ty_column(&to)?;
                let shape = relation_shape(&self.ty, &self.a)?;
                Ok(has_many_condition(&column, &shape.to_col))
            }
            RelationTy::ManyToMany => many_to_many_reachable_ids(&self.a),
            RelationTy::BelongsTo | RelationTy::HasOne => {
                let msg = "count is only available for has_many and many_to_many relations, this should be checked already in RelationAttr validate";
                Err(self.a.inner.syn_err(msg))
            }
        }
    }
}

impl AttrDebug for GenRelationCount {
    fn attr_debug(&self) -> String {
        self.a.inner.attr_debug()
    }
    fn span(&self) -> Span {
        self.a.inner.span
    }
}

impl VirtualResolverFn for GenRelationCount {
    fn sql_dep(&self) -> SynRes<Vec<String>> {
        Ok(vec!["id".to_owned()])
    }
}

impl ResolverFn for GenRelationCount {
    fn name(&self) -> SynRes<Ts2> {
        let base = self.a.name()?.to_string();
        format!("{base}_count").ts2_or_err()
    }
    fn gql_name(&self) -> SynRes<String> {
        let gql_base = self.a.name()?.to_string().to_lower_camel_case();
        Ok(format!("{gql_base}_count"))
    }
    fn tx(&self) -> bool {
        false
    }
    fn inputs(&self) -> SynRes<Ts2> {
        let filter = ty_filter(self.a.to()?)?;
        let inputs = quote! {
            filter: Option<#filter>,
        };
        Ok(push_include_deleted(inputs, self.a.include_deleted))
    }
    fn output(&self) -> SynRes<Ts2> {
        Ok(quote!(u64))
    }
    fn body(&self) -> SynRes<Ts2> {
        let model = self.a.to()?;
        let filter_ty = ty_filter(&model)?;
        let authz_row_filter = gen_authz_row_filter(&filter_ty, self.a.authz_row);
        let include_deleted = get_include_deleted(self.a.include_deleted);
        let extra_cond = self.extra_cond()?;

        Ok(quote! {
            if let Some(id) = self.id.clone() {
                let tx = &*ctx.tx().await?;
                #model::gql_count(
                    tx,
                    filter,
                    #include_deleted,
                    None,
                    Some(#extra_cond),
                    #authz_row_filter,
                )
                .await?
            } else {
                0
            }
        })
    }
}
