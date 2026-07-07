use crate::prelude::*;

/// Proc-macro entry for #[one_resolver], wraps the annotated fn body into an
/// async resolver returning Option of the target model's gql filter type.
pub fn gen_one_resolver(attr: TokenStream, item: TokenStream) -> TokenStream {
    let a = parse_macro_input!(attr as AttrParse);
    let ifn = parse_macro_input!(item as ItemFn);
    try_gen_one_resolver(a, &ifn).unwrap_or_else(|e| e.to_compile_error().into())
}

/// Parsed arguments of #[one_resolver(Model, parent = "Parent")], model is the
/// target entity the resolver loads, parent fixes the generated resolver's
/// parent type, when omitted it stays generic over any GqlModel.
#[field_names]
pub struct OneResolverAttr {
    #[field_names(skip)]
    pub model: String,
    pub parent: Option<String>,
}
impl TryFrom<Attr> for OneResolverAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        Ok(Self {
            model: a.model_from_first_path()?,
            parent: a.str(Self::FIELD_PARENT)?,
        })
    }
}
impl AttrValidate for OneResolverAttr {
    fn attr_fields(a: &Attr) -> Vec<String> {
        Self::FIELDS
            .iter()
            .copied()
            .map(|f| f.to_owned())
            .chain(a.first_path.iter().cloned())
            .collect()
    }
}

fn try_gen_one_resolver(a: AttrParse, ifn: &ItemFn) -> SynRes<TokenStream> {
    let a = Attr::from_proc_macro("one_resolver", a)?.try_into_with_validate::<OneResolverAttr>()?;
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
            include_deleted: Option<&bool>,
        ) -> Res<Option<#filter>>
        where
            D: ConnectionTrait,
            #parent_where
        {
            Ok(Some(#body))
        }
    };
    Ok(r.into())
}
