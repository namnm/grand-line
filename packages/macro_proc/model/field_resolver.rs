use crate::prelude::*;

pub fn gen_field_resolver(attr: TokenStream, item: TokenStream) -> TokenStream {
    let a = parse_macro_input!(attr as AttrParse);
    let ifn = parse_macro_input!(item as ItemFn);
    try_gen_field_resolver(a, &ifn).unwrap_or_else(|e| e.to_compile_error().into())
}

#[field_names]
pub struct FieldResolverAttr {
    pub parent: Option<String>,
}
impl TryFrom<Attr> for FieldResolverAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        Ok(Self {
            parent: a.str(Self::FIELD_PARENT)?,
        })
    }
}
impl AttrValidate for FieldResolverAttr {
    fn attr_fields(_a: &Attr) -> Vec<String> {
        Self::FIELDS.iter().copied().map(|f| f.to_owned()).collect()
    }
}

/// Extract the fn's declared output type - field_resolver, unlike many_resolver/count_resolver,
/// wraps whatever the user declares rather than generating the return type itself.
fn extract_output(sig: &Signature) -> SynRes<Ts2> {
    match &sig.output {
        ReturnType::Type(_, ty) => Ok(ty.to_token_stream()),
        ReturnType::Default => {
            let msg = "should declare an output type, e.g. fn f() -> SomeType";
            Err(SynErr::new(sig.span(), msg))
        }
    }
}

fn try_gen_field_resolver(a: AttrParse, ifn: &ItemFn) -> SynRes<TokenStream> {
    let a = Attr::from_proc_macro("field_resolver", a)?.try_into_with_validate::<FieldResolverAttr>()?;
    ensure_no_inputs(&ifn.sig)?;
    let output = extract_output(&ifn.sig)?;

    let f = &ifn.sig.ident;
    let vis = &ifn.vis;
    let stmts = &ifn.block.stmts;
    let body = quote!(#(#stmts)*);
    let body = ensure_default_tail(body)?;

    let (parent_generics, parent_ty, parent_where) = parent_ty_parts(a.parent.as_ref())?;

    let r = quote! {
        #vis async fn #f<#parent_generics>(
            parent: &#parent_ty,
            ctx: &Context<'_>,
        ) -> Res<#output>
        where
            #parent_where
        {
            Ok(#body)
        }
    };
    Ok(r.into())
}
