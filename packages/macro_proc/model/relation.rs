use crate::prelude::*;

pub struct GenRelation {
    pub ty: RelationTy,
    pub a: RelationAttr,
    pub field_attrs: Vec<Attribute>,
}

impl GenRelation {
    fn sql_dep_str(&self) -> SynRes<String> {
        Ok(match self.ty {
            RelationTy::BelongsTo => self.a.key_str()?,
            RelationTy::HasOne => "id".to_owned(),
            RelationTy::HasMany => "id".to_owned(),
            RelationTy::ManyToMany => "id".to_owned(),
        })
    }
    fn input_one(&self) -> Ts2 {
        let mut inputs = quote!();
        inputs = push_include_deleted(inputs, self.a.include_deleted);
        inputs
    }
    fn input_many(&self) -> SynRes<Ts2> {
        let to = self.a.to()?;
        let filter = ty_filter(&to)?;
        let order_by = ty_order_by(&to)?;
        let inputs = quote! {
            filter: Option<#filter>,
            order_by: Option<Vec<#order_by>>,
            page: Option<Pagination>,
        };
        let inputs = push_include_deleted(inputs, self.a.include_deleted);
        Ok(inputs)
    }

    fn output_one(&self) -> SynRes<Ts2> {
        let to = self.a.gql_to()?;
        Ok(quote!(Option<#to>))
    }
    fn output_many(&self) -> SynRes<Ts2> {
        let to = self.a.gql_to()?;
        Ok(quote!(Vec<#to>))
    }

    fn body_utils(&self, r: &Ts2, vec: bool) -> SynRes<Ts2> {
        let sql_dep = self.sql_dep_str()?.ts2_or_err()?;
        let none = if vec {
            quote!(vec![])
        } else {
            quote!(None)
        };

        let r = quote! {
            if let Some(id) = self.#sql_dep.clone() {
                let tx = &*ctx.tx().await?;
                #r
            } else {
                #none
            }
        };
        Ok(r)
    }

    fn col(&self) -> SynRes<Ts2> {
        Ok(relation_shape(&self.ty, &self.a)?.to_col)
    }

    fn body_one(&self) -> SynRes<Ts2> {
        let column = self.column()?;
        let col = self.col()?;

        let model = self.a.to()?;
        let authz_row_filter = gen_authz_row_filter(&ty_filter(&model)?, self.a.authz_row);
        let include_deleted = get_include_deleted(self.a.include_deleted);

        let r = quote! {
            #model::gql_load(
                ctx,
                tx,
                #column::#col,
                id,
                #authz_row_filter,
                #include_deleted,
            )
            .await?
        };
        self.body_utils(&r, false)
    }

    fn body_many(&self, extra_cond: &Ts2) -> SynRes<Ts2> {
        let model = self.a.to()?;
        let filter = ty_filter(&model)?;
        let order_by = ty_order_by(&model)?;
        let include_deleted = get_include_deleted(self.a.include_deleted);
        let authz_row_filter = gen_authz_row_filter(&filter, self.a.authz_row);

        let (resolver, filter_extra, order_by_default) = if let Some(f) = &self.a.resolver {
            let filter_extra = unique_ident();
            let order_by_default = unique_ident();
            let resolver = quote! {
                let (#filter_extra, #order_by_default): (Option<#filter>, Option<Vec<#order_by>>) = #f(
                    self,
                    ctx,
                    tx,
                    &filter,
                    &order_by,
                    &page,
                    &#include_deleted,
                )
                .await?;
            };
            (resolver, filter_extra, order_by_default)
        } else {
            (quote!(), quote!(None), quote!(None))
        };

        let r = quote! {
            #resolver
            #model::gql_search(
                ctx,
                tx,
                filter,
                order_by,
                page,
                #include_deleted,
                #filter_extra,
                #order_by_default,
                Some(#extra_cond),
                #authz_row_filter,
            )
            .await?
        };
        self.body_utils(&r, true)
    }

    fn body_has_many(&self) -> SynRes<Ts2> {
        let column = self.column()?;
        let col = self.col()?;
        let extra_cond = has_many_condition(&column, &col);
        self.body_many(&extra_cond)
    }

    fn body_many_to_many(&self) -> SynRes<Ts2> {
        let extra_cond = many_to_many_reachable_ids(&self.a)?;
        self.body_many(&extra_cond)
    }

    fn column(&self) -> SynRes<Ts2> {
        ty_column(self.a.to()?)
    }
}

impl VirtualResolverFn for GenRelation {
    fn sql_dep(&self) -> SynRes<Vec<String>> {
        Ok(vec![self.sql_dep_str()?])
    }
}
impl AttrDebug for GenRelation {
    fn attr_debug(&self) -> String {
        self.a.inner.attr_debug()
    }
    fn span(&self) -> Span {
        self.a.inner.span
    }
}

impl ResolverFn for GenRelation {
    fn name(&self) -> SynRes<Ts2> {
        self.a.name()
    }
    fn gql_name(&self) -> SynRes<String> {
        let gql_name_override = attr_graphql_info(&self.field_attrs).0;
        if let Some(n) = gql_name_override {
            return Ok(n);
        }
        Ok(self.name()?.to_string().to_lower_camel_case())
    }
    fn docs(&self) -> Vec<String> {
        attr_docs(&self.field_attrs)
    }
    fn extra_graphql(&self) -> Ts2 {
        attr_graphql_info(&self.field_attrs).1
    }
    fn inputs(&self) -> SynRes<Ts2> {
        match self.ty {
            RelationTy::BelongsTo => Ok(self.input_one()),
            RelationTy::HasOne => Ok(self.input_one()),
            RelationTy::HasMany => self.input_many(),
            RelationTy::ManyToMany => self.input_many(),
        }
    }
    fn output(&self) -> SynRes<Ts2> {
        match self.ty {
            RelationTy::BelongsTo => self.output_one(),
            RelationTy::HasOne => self.output_one(),
            RelationTy::HasMany => self.output_many(),
            RelationTy::ManyToMany => self.output_many(),
        }
    }
    fn body(&self) -> SynRes<Ts2> {
        match self.ty {
            RelationTy::BelongsTo => self.body_one(),
            RelationTy::HasOne => self.body_one(),
            RelationTy::HasMany => self.body_has_many(),
            RelationTy::ManyToMany => self.body_many_to_many(),
        }
    }
}

/// Register one relation field: its main resolver, its `_some`/`_none`/`_every` filter
/// fields, and its `_count` resolver if enabled. The single entry point so a relation
/// can not end up with only some of these wired in.
pub fn register_relation(
    ty: &RelationTy,
    raw_attr: &Attr,
    field_attrs: Vec<Attribute>,
    filter_struk: &mut Vec<Ts2>,
    filter_query: &mut Vec<Ts2>,
    virtual_resolvers: &mut Vec<Box<dyn VirtualResolverFn>>,
) -> SynRes<()> {
    let g = GenRelation {
        ty: ty.clone(),
        a: raw_attr.clone().try_into_with_validate()?,
        field_attrs,
    };
    g.push_filter(filter_struk, filter_query)?;
    let is_to_many = matches!(g.ty, RelationTy::HasMany | RelationTy::ManyToMany);
    let count = g.a.count;
    virtual_resolvers.push(Box::new(g));

    if is_to_many && count {
        let gc = GenRelationCount {
            ty: ty.clone(),
            a: raw_attr.clone().try_into_with_validate()?,
        };
        virtual_resolvers.push(Box::new(gc));
    }
    Ok(())
}
