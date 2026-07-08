use crate::prelude::*;

pub fn gen_model(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttrParse);
    let item = parse_macro_input!(item as ItemStruct);
    try_gen_model(attr, item).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_model(attr: AttrParse, mut item: ItemStruct) -> SynRes<TokenStream> {
    let a = attr.into_inner::<ModelAttr>("model")?;

    // ------------------------------------------------------------------------
    // insert built-in fields: id, created_at, updated_at...
    let model = item.ident.to_token_stream();
    let mut fields = if let Fields::Named(f) = item.fields {
        f.named
    } else {
        let msg = format!("{model} struct should be named fields");
        return Err(a.inner.syn_err(&msg));
    };
    fields.insert(
        0,
        quote! {
            #[sea_orm(primary_key, column_type = "String(StringLen::N(26))", auto_increment = false)]
            pub id: String
        }
        .field_or_err()?,
    );
    if a.created_at {
        fields.push(
            quote! {
                pub created_at: DateTimeUtc
            }
            .field_or_err()?,
        );
        if a.by_id {
            fields.push(
                quote! {
                    pub created_by_id: Option<String>
                }
                .field_or_err()?,
            );
        }
    }
    if a.updated_at {
        fields.push(
            quote! {
                pub updated_at: Option<DateTimeUtc>
            }
            .field_or_err()?,
        );
        if a.by_id {
            fields.push(
                quote! {
                    pub updated_by_id: Option<String>
                }
                .field_or_err()?,
            );
        }
    }
    if a.deleted_at {
        fields.push(
            quote! {
                pub deleted_at: Option<DateTimeUtc>
            }
            .field_or_err()?,
        );
        if a.by_id {
            fields.push(
                quote! {
                    pub deleted_by_id: Option<String>
                }
                .field_or_err()?,
            );
        }
    }

    // ------------------------------------------------------------------------
    // parse macro attributes, extract and validate fields
    let model_str = model.to_string();
    let ModelFieldsAttr {
        defaults,
        virtuals,
        exprs,
        gql_fields,
        sql_fields,
    } = model_fields_attr(&model_str, &fields)?;

    // ------------------------------------------------------------------------
    // get the original model name, and set the new name that sea_orm requires
    // get the original model name in snake case for sql table, non-plural
    item.ident = format_ident!("Model");
    item.fields = Fields::Named(sql_fields);
    let module = model.to_string().to_snake_case().ts2_or_err()?;
    let sql = ty_sql(&model)?;
    let gql = ty_gql(&model)?;
    let column = ty_column(&model)?;
    let active_model = ty_active_model(&model)?;
    let gql_alias = model.to_string();
    let sql_alias = model.to_string().to_snake_case();

    // ------------------------------------------------------------------------
    // model built in cols
    let col_id = quote!(Column::Id);
    let col_opt = |skip: bool, col: Ts2| -> Ts2 {
        if skip {
            quote!(None)
        } else {
            quote!(Some(#col))
        }
    };
    let col_created_at = col_opt(!a.created_at, quote!(Column::CreatedAt));
    let col_updated_at = col_opt(!a.updated_at, quote!(Column::UpdatedAt));
    let col_deleted_at = col_opt(!a.deleted_at, quote!(Column::DeletedAt));
    let col_created_by_id = col_opt(!a.created_at || !a.by_id, quote!(Column::CreatedById));
    let col_updated_by_id = col_opt(!a.updated_at || !a.by_id, quote!(Column::UpdatedById));
    let col_deleted_by_id = col_opt(!a.deleted_at || !a.by_id, quote!(Column::DeletedById));

    // ------------------------------------------------------------------------
    // active model default
    let mut self_am_defaults = vec![];
    for a in defaults {
        let mut raw_str = a.raw()?;
        if (raw_str.starts_with('"') && raw_str.ends_with('"')) || (raw_str.starts_with("r#") && raw_str.ends_with('#'))
        {
            raw_str += ".to_owned()";
        }
        let raw = raw_str.ts2_or_err()?;
        let name = a.field_name()?.ts2_or_err()?;
        self_am_defaults.push(quote! {
            if !matches!(self.#name, Set(_)) {
                self.#name = Set(#raw);
            }
        });
    }

    // ------------------------------------------------------------------------
    // active model get/set
    let am_field = |skip: bool, field: &str| -> SynRes<(Ts2, Ts2)> {
        if skip {
            let r = (quote!(NotSet), quote!(self));
            return Ok(r);
        }
        let f = field.ts2_or_err()?;
        let r = (quote!(self.#f.clone()), quote! { self.#f = Set(v); self });
        Ok(r)
    };
    let (am_get_created_at, am_set_created_at) = am_field(!a.created_at, "created_at")?;
    let (am_get_updated_at, am_set_updated_at) = am_field(!a.updated_at, "updated_at")?;
    let (am_get_deleted_at, am_set_deleted_at) = am_field(!a.deleted_at, "deleted_at")?;
    let (am_get_created_by_id, am_set_created_by_id) = am_field(!a.created_at || !a.by_id, "created_by_id")?;
    let (am_get_updated_by_id, am_set_updated_by_id) = am_field(!a.updated_at || !a.by_id, "updated_by_id")?;
    let (am_get_deleted_by_id, am_set_deleted_by_id) = am_field(!a.deleted_at || !a.by_id, "deleted_by_id")?;

    // ------------------------------------------------------------------------
    // filter / order_by
    let (mut filter_struk, mut filter_query) = (vec![], vec![]);
    let (mut order_by_struk, mut order_by_query) = (vec![], vec![]);
    for (f, a) in &gql_fields {
        if attr_is_gql_skip(a) {
            continue;
        }
        filter(f, &mut filter_struk, &mut filter_query)?;
        order_by(f, &mut order_by_struk, &mut order_by_query)?;
    }
    let filter = ty_filter(&model)?;
    let order_by = ty_order_by(&model)?;
    filter_and_or_not(&filter, &mut filter_struk, &mut filter_query)?;

    // ------------------------------------------------------------------------
    // filter has_deleted_at
    let has_deleted_at = if a.deleted_at {
        quote! {
            !self.deleted_at.is_undefined() ||
            !self.deleted_at_ne.is_undefined() ||
            self.deleted_at_in.is_some() ||
            self.deleted_at_not_in.is_some() ||
            self.deleted_at_gt.is_some() ||
            self.deleted_at_gte.is_some() ||
            self.deleted_at_lt.is_some() ||
            self.deleted_at_lte.is_some()
        }
    } else {
        quote!(false)
    };

    // ------------------------------------------------------------------------
    // gql
    let GqlAttr {
        struk: mut gql_struk,
        struk_fields: mut gql_struk_fields,
        defaults: mut gql_defaults,
        resolver: mut gql_resolver,
        into: gql_into,
        cols: gql_cols,
        select: mut gql_select,
        get_string: gql_get_string,
    } = gql_attr(&gql_fields)?;
    let GqlAttrExprs {
        struk: gql_struk2,
        struk_fields: gql_struk_fields2,
        defaults: gql_defaults2,
        resolver: gql_resolver2,
        select: gql_select2,
        exprs: gql_exprs,
    } = gql_exprs_ts2(&exprs)?;
    gql_struk.extend(gql_struk2);
    gql_struk_fields.extend(gql_struk_fields2);
    gql_defaults.extend(gql_defaults2);
    gql_resolver.extend(gql_resolver2);
    gql_select.extend(gql_select2);

    let gql_into_default = if gql_struk.len() > gql_into.len() {
        quote!(..Default::default())
    } else {
        quote!()
    };

    // ------------------------------------------------------------------------
    // virtual resolvers
    let mut virtual_resolvers = Vec::<Box<dyn VirtualResolverFn>>::new();
    let valid_sql_dep = gql_struk_fields.iter().collect::<HashSet<_>>();
    for (f, attrs) in &virtuals {
        let map = attrs
            .iter()
            .map(|a| (a.attr.clone(), a.clone()))
            .collect::<HashMap<_, _>>();
        for (a, v) in VirtualTy::all()
            .iter()
            .filter_map(|v| map.get(v.to_string().as_str()).map(move |a| (a, v)))
        {
            virtual_resolvers.push(match v {
                VirtualTy::Relation(ty) => Box::new(GenRelation {
                    ty: ty.clone(),
                    a: a.clone().try_into_with_validate()?,
                    field_attrs: f.attrs.clone(),
                }),
                VirtualTy::Resolver => {
                    let g = GenResolver {
                        a: a.clone().try_into_with_validate()?,
                        field_attrs: f.attrs.clone(),
                    };
                    for d in &g.a.sql_dep {
                        if !valid_sql_dep.contains(d) {
                            let msg = format!("can not find column or expr {d}");
                            return Err(g.a.inner.err_by_key("sql_dep", &msg));
                        }
                    }
                    Box::new(g)
                }
                _ => {
                    let msg = "is invalid for virtual resolver";
                    return Err(a.err_by_key(v.as_ref(), msg));
                }
            });
        }
    }

    let GqlAttrVirtuals {
        select: gql_select2,
    } = gql_attr_virtuals(&virtual_resolvers)?;
    gql_select.extend(gql_select2);

    for f in virtual_resolvers {
        gql_resolver.push(f.resolver_fn()?);
    }

    let r = quote! {
        mod #module {
            use super::*;

            #[derive(
                Debug,
                Clone,
                DeriveEntityModel,
            )]
            #[sea_orm(table_name = #sql_alias)]
            #item

            #[derive(
                Debug,
                Clone,
                FromQueryResult,
            )]
            pub struct #gql {
                #(#gql_struk)*
            }
            #[Object(name = #gql_alias)]
            impl #gql {
                #(#gql_resolver)*
            }

            /// ActiveModelBehavior is intentionally empty.
            /// Use the following macros instead to set default values on active models:
            /// #[default], am_create!, am_update!, am_soft_delete!
            /// Other lifecycle related logic should be handled in the service layer
            /// to keep the model clean and focused on data representation.
            impl ActiveModelBehavior for ActiveModel {
            }
            #[derive(Debug, EnumIter, DeriveRelation)]
            /// Relation is intentionally empty.
            /// Relationships are handled via: #[has_one], #[has_many], #[belongs_to], #[many_to_many]
            /// and resolved through the dataloader system, not sea-orm's native relation API.
            pub enum Relation {
            }

            static GQL_COLS: LazyLock<HashMap<&'static str, Column>> = LazyLock::new(|| {
                let mut m = HashMap::new();
                #(#gql_cols)*
                m
            });
            static GQL_EXPRS: LazyLock<HashMap<&'static str, SimpleExpr>> = LazyLock::new(|| {
                let mut m = HashMap::new();
                #(#gql_exprs)*
                m
            });
            static GQL_SELECT: LazyLock<HashMap<&'static str, HashSet<&'static str>>> = LazyLock::new(|| {
                let mut m = HashMap::new();
                #(#gql_select)*
                m
            });

            impl EntityX for Entity {
                type M = Model;
                type A = ActiveModel;
                type C = Column;
                type F = #filter;
                type O = #order_by;
                type G = #gql;
                fn model_name() -> &'static str {
                    #model_str
                }
                fn col_id() -> Self::C {
                    #col_id
                }
                fn col_created_at() -> Option<Self::C> {
                    #col_created_at
                }
                fn col_updated_at() -> Option<Self::C> {
                    #col_updated_at
                }
                fn col_deleted_at() -> Option<Self::C> {
                    #col_deleted_at
                }
                fn col_created_by_id() -> Option<Self::C> {
                    #col_created_by_id
                }
                fn col_updated_by_id() -> Option<Self::C> {
                    #col_updated_by_id
                }
                fn col_deleted_by_id() -> Option<Self::C> {
                    #col_deleted_by_id
                }
                fn gql_cols() -> &'static LazyLock<HashMap<&'static str, Self::C>> {
                    &GQL_COLS
                }
                fn gql_exprs() -> &'static LazyLock<HashMap<&'static str, SimpleExpr>> {
                    &GQL_EXPRS
                }
                fn gql_select() -> &'static LazyLock<HashMap<&'static str, HashSet<&'static str>>> {
                    &GQL_SELECT
                }
            }

            impl ModelX<Entity> for Model {
                fn get_id(&self) -> String {
                    self.id.clone()
                }
                fn into_gql_without_look_ahead(self) -> #gql {
                    #gql {
                        #(#gql_into)*
                        #gql_into_default
                    }
                }
            }

            impl ActiveModelX<Entity> for ActiveModel {
                fn set_defaults(mut self) -> Self {
                    #(#self_am_defaults)*
                    self
                }
                fn get_id(&self) -> ActiveValue<String> {
                    self.id.clone()
                }
                fn set_id(mut self, v: &str) -> Self {
                    self.id = Set(v.to_owned());
                    self
                }
                fn get_created_at(&self) -> ActiveValue<DateTimeUtc> {
                    #am_get_created_at
                }
                fn set_created_at(mut self, v: DateTimeUtc) -> Self {
                    #am_set_created_at
                }
                fn get_updated_at(&self) -> ActiveValue<Option<DateTimeUtc>> {
                    #am_get_updated_at
                }
                fn set_updated_at(mut self, v: Option<DateTimeUtc>) -> Self {
                    #am_set_updated_at
                }
                fn get_deleted_at(&self) -> ActiveValue<Option<DateTimeUtc>> {
                    #am_get_deleted_at
                }
                fn set_deleted_at(mut self, v: Option<DateTimeUtc>) -> Self {
                    #am_set_deleted_at
                }
                fn get_created_by_id(&self) -> ActiveValue<Option<String>> {
                    #am_get_created_by_id
                }
                fn set_created_by_id(mut self, v: Option<String>) -> Self {
                    #am_set_created_by_id
                }
                fn get_updated_by_id(&self) -> ActiveValue<Option<String>> {
                    #am_get_updated_by_id
                }
                fn set_updated_by_id(mut self, v: Option<String>) -> Self {
                    #am_set_updated_by_id
                }
                fn get_deleted_by_id(&self) -> ActiveValue<Option<String>> {
                    #am_get_deleted_by_id
                }
                fn set_deleted_by_id(mut self, v: Option<String>) -> Self {
                    #am_set_deleted_by_id
                }
            }

            impl ColumnX<Entity> for Column {}

            impl GqlModel<Entity> for #gql {
                fn set_id(mut self, v: &str) -> Self {
                    self.id = Some(v.to_owned());
                    self
                }
                fn get_string(&self, col: Column) -> Option<String> {
                    match col {
                        #(#gql_get_string)*
                        _ => None
                    }
                }
            }
            impl Default for #gql {
                fn default() -> Self {
                    Self {
                        #(#gql_defaults)*
                    }
                }
            }

            #[gql_input]
            pub struct #filter {
                #(#filter_struk)*
            }
            impl Filter<Entity> for #filter {
                fn combine_and(a: Self, b: Self) -> Self {
                    Self {
                        and: Some(vec![a, b]),
                        ..Default::default()
                    }
                }
                fn has_deleted_at_without_nested(&self) -> bool {
                    #has_deleted_at
                }
                fn get_and(&self) -> Option<Vec<Self>> {
                    self.and.clone()
                }
                fn get_or(&self) -> Option<Vec<Self>> {
                    self.or.clone()
                }
                fn get_not(&self) -> Option<Self> {
                    self.not.clone().map(|b| *b)
                }
            }
            impl IntoCondition for #filter {
                fn into_condition(self) -> Condition {
                    let this = self.clone();
                    let mut c = Condition::all();
                    #(#filter_query)*
                    c
                }
            }
            /// This should be here since it is conflicted with foreign trait IntoCondition.
            impl ChainSelect<Entity> for #filter {
                fn chain_select(self, q: Select<Entity>) -> Select<Entity> {
                    q.filter(self.into_condition())
                }
            }

            #[gql_enum]
            pub enum #order_by {
                #(#order_by_struk)*
            }
            impl OrderBy<Entity> for #order_by {
                fn conf_default() -> Self {
                    Self::IdDesc
                }
            }
            impl ChainSelect<Entity> for #order_by {
                fn chain_select(self, q: Select<Entity>) -> Select<Entity> {
                    match self {
                        #(#order_by_query)*
                    }
                }
            }
        }
        pub use #module::{
            Model as #sql,
            Entity as #model,
            Column as #column,
            ActiveModel as #active_model,
            #gql,
            #filter,
            #order_by,
        };
    };

    #[cfg(feature = "debug_macro")]
    debug_macro(&model_str, &r);

    Ok(r.into())
}
