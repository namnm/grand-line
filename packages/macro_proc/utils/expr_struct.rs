use crate::prelude::*;

pub fn expr_struct(item: TokenStream, suf: &str, wrap: &str, method: &str) -> TokenStream {
    let struk = match parse_struct(item) {
        Ok(struk) => struk,
        Err(e) => return e.to_compile_error().into(),
    };
    try_expr_struct(&struk, suf, wrap, method).unwrap_or_else(|e| e.to_compile_error().into())
}

fn parse_struct(item: TokenStream) -> SynRes<ExprStruct> {
    let item_ts2 = Into::<Ts2>::into(item.clone());
    let item_with_braces = if item_ts2.to_string().trim().ends_with('}') {
        item
    } else {
        Into::<TokenStream>::into(quote!(#item_ts2{}))
    };
    parse::<ExprStruct>(item_with_braces)
}

fn try_expr_struct(struk: &ExprStruct, suf: &str, wrap: &str, method: &str) -> SynRes<TokenStream> {
    let struk = build_expr_struct(struk, suf, wrap)?;
    let r = if method.is_empty() {
        struk
    } else {
        let method = method.ts2_or_err()?;
        quote!(#struk.#method())
    };
    Ok(r.into())
}

pub fn expr_struct_am_wrapper(item: TokenStream, suf: &str, op_ty: &str) -> TokenStream {
    let item = match parse_struct(item) {
        Ok(item) => item,
        Err(e) => return e.to_compile_error().into(),
    };
    try_expr_struct_am_wrapper(&item, suf, op_ty).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_expr_struct_am_wrapper(item: &ExprStruct, suf: &str, op_ty: &str) -> SynRes<TokenStream> {
    let entity = item.path.get_ident().to_token_stream();
    let struk = build_expr_struct(item, suf, "Set")?;
    let op_ty = op_ty.ts2_or_err()?;

    let r = quote! {
        AmWrapper::<#op_ty, #entity, _>::new(#struk)
    };
    Ok(r.into())
}

// ============================================================================
// Array variant, one model name followed by an array of field-only blocks,
// e.g. am_create_many!(Todo, [{ content: "a" }, { content: "b" }]).
// Each block is re-injected with the model name via quote! and parsed as a
// regular ExprStruct, so build_expr_struct stays the single source of truth.

struct ManyInput {
    model: Ident,
    items: Vec<Ts2>,
}

impl Parse for ManyInput {
    fn parse(input: ParseStream) -> SynRes<Self> {
        let model = input.parse::<Ident>()?;
        input.parse::<Comma>()?;

        let arr;
        bracketed!(arr in input);

        let mut items = vec![];
        while !arr.is_empty() {
            let item;
            braced!(item in arr);
            items.push(item.parse::<Ts2>()?);
            if arr.peek(Comma) {
                arr.parse::<Comma>()?;
            }
        }

        Ok(Self {
            model,
            items,
        })
    }
}

pub fn expr_array_am_wrapper(item: TokenStream, suf: &str, op_ty: &str) -> TokenStream {
    let input = parse_macro_input!(item as ManyInput);
    try_expr_array_am_wrapper(&input, suf, op_ty).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_expr_array_am_wrapper(input: &ManyInput, suf: &str, op_ty_str: &str) -> SynRes<TokenStream> {
    let model = &input.model;
    let op_ty = op_ty_str.ts2_or_err()?;

    let mut wrapped = vec![];
    for fields in &input.items {
        let struk = quote!(#model { #fields });
        let struk = parse2::<ExprStruct>(struk)?;
        let am = build_expr_struct(&struk, suf, "Set")?;
        let am = quote!(AmWrapper::<#op_ty, #model, _>::new(#am));
        wrapped.push(am);
    }

    // Annotate the element type explicitly, an empty array would otherwise leave
    // vec![] with no AmWrapper<T, E, A> expression to infer the type from.
    let items = quote! {
        {
            let v: Vec<AmWrapper<#op_ty, #model, _>> = vec![
                #(#wrapped,)*
            ];
            v
        }
    };

    // Each _many macro returns its own dedicated wrapper (AmCreateMany carries the
    // returning() opt-in, AmUpdateMany / AmSoftDeleteMany just run one op per row).
    let wrapper = match op_ty_str {
        "AmCreate" => quote!(AmCreateMany),
        "AmUpdate" => quote!(AmUpdateMany),
        "AmSoftDelete" => quote!(AmSoftDeleteMany),
        _ => {
            return Err(SynErr::new(
                Span::call_site(),
                format!("unknown am _many op type: {op_ty_str}"),
            ));
        }
    };

    let r = quote!(#wrapper::<#model, _>::new(#items));
    Ok(r.into())
}

fn build_expr_struct(item: &ExprStruct, suf: &str, wrap: &str) -> SynRes<Ts2> {
    let model = item.path.get_ident().to_token_stream();
    let name = format!("{model}{suf}").ts2_or_err()?;

    let rest = item.rest.to_token_stream();
    let rest = if rest.to_string().trim().is_empty() {
        quote!(..Default::default())
    } else {
        quote!(..#rest)
    };

    let mut fields = vec![];
    for f in &item.fields {
        let v = if let Expr::Lit(l) = &f.expr {
            if let Lit::Str(s) = &l.lit {
                let v = s.value();
                quote!(#v.to_owned())
            } else {
                l.to_token_stream()
            }
        } else {
            f.expr.to_token_stream()
        };
        let member = f.member.to_token_stream();
        let wrap = format!("{member}:{wrap}").ts2_or_err()?;
        fields.push(quote!(#wrap(#v)));
    }

    let r = quote! {
        #name {
            #(#fields,)*
            #rest
        }
    };
    Ok(r)
}
