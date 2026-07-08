use crate::prelude::*;
use core::any::type_name;

#[derive(Clone)]
pub struct Attr {
    /// In proc macro, this is empty.
    /// In field, this will be Model.field.
    debug: String,
    /// In proc macro, this is the macro name.
    /// In field, this will be one of `AttrTy`.
    pub attr: String,
    /// Raw args parsed as strings.
    args: HashMap<String, (String, AttrParseTy)>,
    /// Only in proc macro like crud(Model, ...).
    /// The first path will be the model name.
    first_path: Option<String>,
    /// Only in field.
    field: Option<(String, Attribute, Field)>,
    /// Only in attr such as #[default(..)], #[`sql_expr`(..)], etc..
    raw: Option<String>,
    /// Span of the attribute for error reporting.
    pub span: Span,
}

impl Attr {
    fn init(debug: &str, attr: &str, args: Vec<(String, (String, AttrParseTy))>, span: Span) -> SynRes<Self> {
        let mut a = Self {
            debug: debug.to_owned(),
            attr: attr.to_owned(),
            args: HashMap::new(),
            first_path: None,
            field: None,
            raw: None,
            span,
        };
        for (k, v) in args {
            if a.args.contains_key(&k) {
                let msg = "appears more than once";
                return Err(a.err_by_key(&k, msg));
            }
            a.args.insert(k, v);
        }
        Ok(a)
    }

    pub fn from_proc_macro(macro_name: &str, a: AttrParse) -> SynRes<Self> {
        let mut r = Self::init("", macro_name, a.args, Span::call_site())?;
        r.first_path = a.first_path;
        Ok(r)
    }
    pub fn from_ts2(debug: &str, attr: &str, ts: &Ts2) -> SynRes<Self> {
        let span = ts.span();
        let a = AttrParse::from_meta_list_token_stream(ts)?;
        let mut r = Self::init(debug, attr, a.args, span)?;
        r.first_path = a.first_path;
        Ok(r)
    }
    pub fn from_ts2_into<V>(debug: &str, attr: &str, ts: &Ts2) -> SynRes<V>
    where
        V: TryFrom<Self, Error = SynErr> + AttrValidate,
    {
        Self::from_ts2(debug, attr, ts)?.try_into_with_validate()
    }

    pub fn from_field(model: &str, f: &Field, raw: &dyn Fn(&str) -> bool) -> SynRes<Vec<Self>> {
        f.attrs
            .iter()
            .map(|a| Self::from_field_attr(model, f, a, raw))
            .collect::<SynRes<Vec<_>>>()
    }
    fn from_field_attr(model: &str, f: &Field, a: &Attribute, raw: &dyn Fn(&str) -> bool) -> SynRes<Self> {
        let attr = a.path().to_token_stream().to_string();
        let field = f.ident.to_token_stream();
        let debug = format!("{model}.{field}");
        let span = a.span();
        let field_data = Some((model.to_owned(), a.clone(), f.clone()));
        let mut r = if raw(&attr) {
            let mut r = Self::init(&debug, &attr, vec![], span)?;
            r.raw = Some(if let Meta::List(l) = &a.meta {
                l.tokens.to_string()
            } else {
                let msg = format!("raw attr should be meta list #[{attr}(some_value)]");
                return Err(SynErr::new(span, msg));
            });
            r
        } else {
            match &a.meta {
                // #[attr(nested)]
                Meta::List(l) => Self::from_ts2(&debug, &attr, &l.tokens)?,
                // Meta::Path(_) => #[attr] without any nested meta, args should be empty
                // Meta::NameValue(_) => #[attr = some_value] we are not using, args should be empty
                // there are case such as #[doc = "some_value"] then we should not panic
                _ => Self::init(&debug, &attr, vec![], span)?,
            }
        };
        r.field = field_data;
        r.span = span;
        Ok(r)
    }

    pub fn is(&self, attr: &str) -> bool {
        self.attr == attr
    }
    pub fn has(&self, k: &str) -> bool {
        self.args.contains_key(k)
    }

    pub fn model_from_first_path(&self) -> SynRes<String> {
        if let Some(v) = self.first_path.clone() {
            if v != v.to_pascal_case() {
                let msg = format!("model `{v}` is not pascal case");
                return Err(self.syn_err(&msg));
            }
            Ok(v)
        } else {
            let attr = &self.attr;
            let msg = format!("missing model #[{attr}(Model, ...)]");
            Err(self.syn_err(&msg))
        }
    }

    pub fn bool(&self, k: &str) -> SynRes<Option<bool>> {
        match self.args.get(k) {
            Some((_, AttrParseTy::Path)) => Ok(Some(true)),
            Some((v, AttrParseTy::NameValue)) => {
                if v == "false" {
                    Ok(Some(false))
                } else {
                    Err(self.err_invalid_bool(k))
                }
            }
            Some(_) => Err(self.err_invalid_bool(k)),
            None => Ok(None),
        }
    }
    pub fn bool_required(&self, k: &str) -> SynRes<bool> {
        self.bool(k)?.ok_or_else(|| self.err_required(k))
    }
    pub fn bool_should_omit(&self, k: &str) -> SynRes<bool> {
        match self.bool(k)? {
            Some(v) => {
                if !v {
                    let msg = "should omit";
                    return Err(self.err_by_key(k, msg));
                }
                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub fn str(&self, k: &str) -> SynRes<Option<String>> {
        match self.args.get(k) {
            Some((v, AttrParseTy::NameValue)) => parse2::<LitStr>(v.ts2_or_err()?)
                .map(|v| Ok(Some(v.value())))
                .unwrap_or_else(|_| Err(self.err_invalid_string(k))),
            Some(_) => Err(self.err_invalid_string(k)),
            None => Ok(None),
        }
    }
    pub fn str_required(&self, k: &str) -> SynRes<String> {
        self.str(k)?.ok_or_else(|| self.err_required(k))
    }

    pub fn nested(&self, k: &str) -> SynRes<Option<String>> {
        match self.args.get(k) {
            Some((v, AttrParseTy::List)) => Ok(Some(v.to_owned())),
            Some(_) => Err(self.err_invalid_nested(k)),
            None => Ok(None),
        }
    }
    pub fn nested_required(&self, k: &str) -> SynRes<String> {
        self.nested(k)?.ok_or_else(|| self.err_required(k))
    }
    pub fn nested_into<V>(&self, k: &str) -> SynRes<Option<V>>
    where
        V: TryFrom<Self, Error = SynErr> + AttrValidate,
    {
        match self.nested(k)? {
            Some(v) => Ok(Some(Self::from_ts2_into(&self.attr_debug(), k, &v.ts2_or_err()?)?)),
            None => Ok(None),
        }
    }

    pub fn nested_with_path(&self, k: &str) -> SynRes<Option<(bool, String)>> {
        match self.args.get(k) {
            Some((v, AttrParseTy::Path)) => Ok(Some((true, v.to_owned()))),
            _ => Ok(self.nested(k)?.map(|v| (false, v))),
        }
    }
    pub fn nested_with_path_required(&self, k: &str) -> SynRes<(bool, String)> {
        self.nested_with_path(k)?.ok_or_else(|| self.err_required(k))
    }
    pub fn nested_with_path_into<V>(&self, k: &str) -> SynRes<Option<(bool, V)>>
    where
        V: TryFrom<Self, Error = SynErr> + AttrValidate,
    {
        match self.nested_with_path(k)? {
            Some((path, v)) => Ok(Some((
                path,
                if path {
                    Self::init(&self.attr_debug(), k, vec![], self.span)?.try_into_with_validate()?
                } else {
                    Self::from_ts2_into(&self.attr_debug(), k, &v.ts2_or_err()?)?
                },
            ))),
            None => Ok(None),
        }
    }

    pub fn parse<V>(&self, k: &str) -> SynRes<Option<V>>
    where
        V: FromStr,
    {
        match self.args.get(k) {
            Some((v, AttrParseTy::NameValue)) => v.parse::<V>().map(|v| Ok(Some(v))).unwrap_or_else(|_| {
                let t = type_name::<V>();
                let msg = format!("cannot parse `{v}` as {t}");
                Err(self.err_by_key(k, &msg))
            }),
            Some(_) => {
                let msg = format!("should be `{k} = some_value`");
                Err(self.err_by_key(k, &msg))
            }
            None => Ok(None),
        }
    }
    pub fn parse_required<V>(&self, k: &str) -> SynRes<V>
    where
        V: FromStr,
    {
        self.parse(k)?.ok_or_else(|| self.err_required(k))
    }

    fn field(&self) -> SynRes<(String, Attribute, Field)> {
        self.field.clone().ok_or_else(|| {
            let msg = "field: None (programmer error)";
            SynErr::new(self.span, msg)
        })
    }
    pub fn field_model(&self) -> SynRes<String> {
        Ok(self.field()?.0)
    }
    pub fn field_attr(&self) -> SynRes<Attribute> {
        Ok(self.field()?.1)
    }
    pub fn field_name(&self) -> SynRes<String> {
        Ok(self.field()?.2.ident.to_token_stream().to_string())
    }
    pub fn field_ty(&self) -> SynRes<String> {
        Ok(self.field()?.2.ty.to_token_stream().to_string())
    }

    pub fn raw(&self) -> SynRes<String> {
        self.raw.clone().ok_or_else(|| {
            let msg = "raw: None (programmer error)";
            SynErr::new(self.span, msg)
        })
    }

    pub fn try_into_with_validate<V>(self) -> SynRes<V>
    where
        V: TryFrom<Self, Error = SynErr> + AttrValidate,
    {
        let attrs = V::attr_fields(&self);
        let map = attrs.iter().collect::<HashSet<_>>();
        for (k, _) in self.args.clone() {
            if !map.contains(&k) {
                return Err(self.err_invalid(&k, &attrs));
            }
        }
        self.try_into()
    }

    pub fn err_required(&self, k: &str) -> SynErr {
        let msg = "is required";
        self.err_by_key(k, msg)
    }
    pub fn err_invalid(&self, k: &str, valid: &[String]) -> SynErr {
        let valid = valid.join(", ");
        let msg = format!("is not valid here, should be one of: {valid}");
        self.err_by_key(k, &msg)
    }
    pub fn err_invalid_bool(&self, k: &str) -> SynErr {
        let msg = format!("should be `{k}` for true, or `{k} = false` for false");
        self.err_by_key(k, &msg)
    }
    pub fn err_invalid_string(&self, k: &str) -> SynErr {
        let msg = format!(r#"should be `{k} = "some_value"` for string"#);
        self.err_by_key(k, &msg)
    }
    pub fn err_invalid_nested(&self, k: &str) -> SynErr {
        let msg = format!("should be `{k}(some_value)` for nested");
        self.err_by_key(k, &msg)
    }
    pub fn err_by_key(&self, k: &str, msg: &str) -> SynErr {
        let msg = format!("key `{k}` {msg}");
        self.syn_err(&msg)
    }
}

impl AttrDebug for Attr {
    fn attr_debug(&self) -> String {
        let Self {
            attr,
            debug,
            ..
        } = &self;
        if debug.is_empty() {
            format!("macro `{attr}`:")
        } else {
            format!("{debug} attr `{attr}`:")
        }
    }
    fn span(&self) -> Span {
        self.span
    }
}

pub trait AttrValidate {
    fn attr_fields(a: &Attr) -> Vec<String>;
}
