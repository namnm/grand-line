use crate::prelude::*;

/// Which syn::Meta variant an attribute argument was parsed from.
#[derive(Clone, Eq, PartialEq)]
pub enum AttrParseTy {
    Path,
    NameValue,
    List,
}

/// Only in proc macro. For example with proc_macro(k, k1=v1, k2=v2)
/// it will only pass the nested part k, k1=v1, k2=v2 to this impl.
pub struct AttrParse {
    /// Parsed (key, (raw value, parsed type)) pairs, in attribute order.
    pub args: Vec<(String, (String, AttrParseTy))>,
    /// Only in proc macro #crud[Model, ...].
    /// The first path will be the model name.
    pub first_path: Option<String>,
}

impl AttrParse {
    /// Wraps self as an Attr for the given proc-macro name, then validates
    /// and converts it into A.
    pub fn into_inner<A>(self, macro_name: &str) -> SynRes<A>
    where
        A: TryFrom<Attr, Error = SynErr> + AttrValidate,
    {
        Attr::from_proc_macro(macro_name, self)?.try_into_with_validate()
    }
    /// Parses a parenthesized meta list token stream, erroring if it is empty.
    pub fn from_meta_list_token_stream(ts: &Ts2) -> SynRes<Self> {
        if ts.to_string().trim().is_empty() {
            let msg = "empty meta list ()";
            return Err(SynErr::new(Span::call_site(), msg));
        }
        let metas = Punctuated::<Meta, Comma>::parse_terminated
            .parse2(ts.clone())
            .map_err(|e| {
                let msg = format!("failed to parse meta list from token stream {ts}: {e}");
                SynErr::new(e.span(), msg)
            })?
            .into_iter()
            .collect();
        Ok(Self::from_meta_list(metas))
    }
    /// Builds Self from already-parsed metas, recording the first bare path
    /// argument (if any) as first_path.
    pub fn from_meta_list(metas: Vec<Meta>) -> Self {
        let mut args = Vec::new();
        let mut first = true;
        let mut first_path = None;
        for m in metas {
            let (k, v, ty);
            match m {
                Meta::Path(m) => {
                    k = m.get_ident().to_token_stream().to_string();
                    v = String::new();
                    ty = AttrParseTy::Path;
                }
                Meta::NameValue(m) => {
                    k = m.path.get_ident().to_token_stream().to_string();
                    v = m.value.to_token_stream().to_string();
                    ty = AttrParseTy::NameValue;
                }
                Meta::List(m) => {
                    k = m.path.get_ident().to_token_stream().to_string();
                    v = m.tokens.to_string();
                    ty = AttrParseTy::List;
                }
            }
            if first && ty == AttrParseTy::Path {
                first_path = Some(k.clone());
            }
            args.push((k, (v, ty)));
            first = false;
        }
        Self {
            args,
            first_path,
        }
    }
}

impl Parse for AttrParse {
    fn parse(input: ParseStream) -> Result<Self> {
        let metas = Punctuated::<Meta, Comma>::parse_terminated(input)?
            .into_iter()
            .collect();
        let a = Self::from_meta_list(metas);
        Ok(a)
    }
}
