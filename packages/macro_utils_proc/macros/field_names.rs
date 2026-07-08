use crate::prelude::*;

pub fn gen_field_names(_: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);
    try_gen_field_names(item).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_field_names(mut item: ItemStruct) -> SynRes<TokenStream> {
    let name = item.ident.to_token_stream();
    let name_span = item.ident.span();
    let mut fields = vec![];
    let mut idents = vec![];

    for mut f in if let Fields::Named(f) = item.fields {
        f.named
    } else {
        let msg = format!("{name} struct should be named fields");
        return Err(SynErr::new(name_span, msg));
    } {
        let attrs = Attr::from_field(&name.to_string(), &f, &|_| false)?;
        if let Some(a) = attrs.iter().find(|a| a.is("field_names")) {
            f.attrs = attrs
                .iter()
                .filter(|b| b.attr != a.attr)
                .map(|b| b.field_attr())
                .collect::<SynRes<Vec<_>>>()?;
            let a = a.clone().try_into_with_validate::<FieldNamesAttr>()?;
            if a.virt {
                if f.to_token_stream().to_string().starts_with("pub ") {
                    return Err(a.inner.syn_err("virtual field should not be public"));
                }
                if f.ty.to_token_stream().to_string() != "!" {
                    return Err(a.inner.syn_err("virtual field type should be `!`"));
                }
            }
            if !a.skip {
                idents.push(f.ident.to_token_stream());
            }
            if !a.virt {
                fields.push(f);
            }
        } else {
            idents.push(f.ident.to_token_stream());
            fields.push(f);
        }
    }

    item.fields = Fields::Named(FieldsNamed {
        named: Punctuated::from_iter(fields),
        brace_token: Default::default(),
    });

    let mut all = vec![];
    let mut impls = vec![];
    for f in idents {
        let f_str = f.to_string();
        all.push(quote! {
            #f_str,
        });
        let f = format!("FIELD_{f}").to_shouty_snake_case().ts2_or_err()?;
        impls.push(quote! {
            pub const #f: &'static str = #f_str;
        });
    }
    let l = all.len();

    Ok(quote! {
        #item
        impl #name {
            pub const FIELDS: [&'static str; #l] = [#(#all)*];
            #(#impls)*
        }
    }
    .into())
}

struct FieldNamesAttr {
    skip: bool,
    virt: bool,
    pub inner: Attr,
}
impl TryFrom<Attr> for FieldNamesAttr {
    type Error = SynErr;
    fn try_from(a: Attr) -> SynRes<Self> {
        Ok(Self {
            skip: a.bool("skip")?.unwrap_or(false),
            virt: a.bool("virt")?.unwrap_or(false),
            inner: a,
        })
    }
}
impl AttrValidate for FieldNamesAttr {
    fn attr_fields(_: &Attr) -> Vec<String> {
        ["skip", "virt"].map(|f| f.to_owned()).to_vec()
    }
}
