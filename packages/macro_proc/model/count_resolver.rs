use crate::prelude::*;

/// Proc-macro entry for #[count_resolver], wraps the annotated fn body into an
/// async resolver returning a Count for the target model.
pub fn gen_count_resolver(attr: TokenStream, item: TokenStream) -> TokenStream {
    let a = parse_macro_input!(attr as AttrParse);
    let ifn = parse_macro_input!(item as ItemFn);
    try_gen_count_resolver(a, &ifn).unwrap_or_else(|e| e.to_compile_error().into())
}

/// Parsed arguments of #[count_resolver(Model, parent = "Parent")], model is the
/// target entity the resolver counts, parent fixes the generated resolver's
/// parent type, when omitted it stays generic over any GqlModel.
#[field_names]
pub struct CountResolverAttr {
    #[field_names(skip)]
    pub model: String,
    pub parent: Option<String>,
}
impl TryFrom<Attr> for CountResolverAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        Ok(Self {
            model: a.model_from_first_path()?,
            parent: a.str(Self::FIELD_PARENT)?,
        })
    }
}
impl AttrValidate for CountResolverAttr {
    fn attr_fields(a: &Attr) -> Vec<String> {
        Self::FIELDS
            .iter()
            .copied()
            .map(|f| f.to_owned())
            .chain(a.first_path.iter().cloned())
            .collect()
    }
}

fn try_gen_count_resolver(a: AttrParse, ifn: &ItemFn) -> SynRes<TokenStream> {
    let a = Attr::from_proc_macro("count_resolver", a)?.try_into_with_validate::<CountResolverAttr>()?;
    ensure_no_inputs(&ifn.sig)?;
    ensure_no_output(&ifn.sig)?;

    let f = &ifn.sig.ident;
    let vis = &ifn.vis;
    let stmts = &ifn.block.stmts;
    let body = quote!(#(#stmts)*);
    let body = ensure_default_tail(body)?;

    let (parent_generics, parent_ty, parent_where) = parent_ty_parts(a.parent.as_ref())?;
    let filter = ty_filter(&a.model)?;

    let r = quote! {
        #vis async fn #f<D, #parent_generics>(
            parent: &#parent_ty,
            ctx: &Context<'_>,
            tx: &D,
            filter: Option<&#filter>,
            include_deleted: Option<&bool>,
        ) -> Res<Count>
        where
            D: ConnectionTrait,
            #parent_where
        {
            Ok(#body)
        }
    };
    Ok(r.into())
}
