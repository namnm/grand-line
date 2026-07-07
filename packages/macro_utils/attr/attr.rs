use crate::prelude::*;
use core::any::type_name;

/// Parsed representation of a single attribute, from either a proc-macro
/// invocation or a struct field, with typed accessors for its arguments.
#[derive(Clone)]
pub struct Attr {
    /// In proc macro, this is empty.
    /// In field, this will be Model.field.
    pub debug: String,
    /// In proc macro, this is the macro name.
    /// In field, this will be one of AttrTy.
    pub attr: String,
    /// Raw args parsed as strings.
    pub args: HashMap<String, (String, AttrParseTy)>,
    /// Only in proc macro like crud(Model, ...).
    /// The first path will be the model name.
    pub first_path: Option<String>,
    /// Only in field.
    pub field: Option<(String, Attribute, Field)>,
    /// Only in attr such as #[default(..)], #[sql_expr(..)], etc..
    pub raw: Option<String>,
    /// Span of the attribute for error reporting.
    pub span: Span,
}

impl Attr {
    // ---------------------------------------------------------------------------
    // Construction from proc-macro or field attributes
    // ---------------------------------------------------------------------------

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

    /// Builds an Attr from a proc-macro's own arguments, e.g. #[crud(Model, ...)].
    /// The span defaults to the call site since there is no attribute token
    /// stream to point at.
    pub fn from_proc_macro(macro_name: &str, a: AttrParse) -> SynRes<Self> {
        let mut r = Self::init("", macro_name, a.args, Span::call_site())?;
        r.first_path = a.first_path;
        Ok(r)
    }
    /// Builds an Attr by parsing a token stream as a meta list, e.g. the
    /// contents of #[attr(k, k1 = v1)].
    pub fn from_ts2(debug: &str, attr: &str, ts: &Ts2) -> SynRes<Self> {
        let span = ts.span();
        let a = AttrParse::from_meta_list_token_stream(ts)?;
        let mut r = Self::init(debug, attr, a.args, span)?;
        r.first_path = a.first_path;
        Ok(r)
    }
    /// Like from_ts2, then validates and converts the result into V.
    pub fn from_ts2_into<V>(debug: &str, attr: &str, ts: &Ts2) -> SynRes<V>
    where
        V: TryFrom<Self, Error = SynErr> + AttrValidate,
    {
        Self::from_ts2(debug, attr, ts)?.try_into_with_validate()
    }

    /// Builds one Attr per attribute on a struct field. When raw(attr_name)
    /// returns true, the attribute's tokens are stored as-is in Attr::raw
    /// instead of being parsed as a meta list.
    pub fn from_field<F>(model: &str, f: &Field, raw: F) -> SynRes<Vec<Self>>
    where
        F: Fn(&str) -> bool,
    {
        f.attrs
            .iter()
            .map(|a| Self::from_field_attr(model, f, a, &raw))
            .collect::<SynRes<Vec<_>>>()
    }
    fn from_field_attr<F>(model: &str, f: &Field, a: &Attribute, raw: &F) -> SynRes<Self>
    where
        F: Fn(&str) -> bool,
    {
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

    // ---------------------------------------------------------------------------
    // Presence and model-name checks
    // ---------------------------------------------------------------------------

    pub fn is(&self, attr: &str) -> bool {
        self.attr == attr
    }
    pub fn has(&self, k: &str) -> bool {
        self.args.contains_key(k)
    }

    /// Returns first_path as the model name, erroring if it is missing or
    /// not PascalCase.
    pub fn model_from_first_path(&self) -> SynRes<String> {
        if let Some(v) = self.first_path.clone() {
            if v != v.to_pascal_case() {
                let msg = format!("model {v} is not pascal case");
                return Err(self.syn_err(&msg));
            }
            Ok(v)
        } else {
            let attr = &self.attr;
            let msg = format!("missing model #[{attr}(Model, ...)]");
            Err(self.syn_err(&msg))
        }
    }

    // ---------------------------------------------------------------------------
    // Typed argument accessors
    // ---------------------------------------------------------------------------

    /// Reads k as a bool: None if absent, Some(true) for a bare flag #[k],
    /// Some(false) for k = false, error for any other form.
    pub fn bool(&self, k: &str) -> SynRes<Option<bool>> {
        let r = match self.args.get(k) {
            Some((_, AttrParseTy::Path)) => Some(true),
            Some((v, AttrParseTy::NameValue)) => {
                if v == "false" {
                    Some(false)
                } else {
                    return Err(self.err_invalid_bool(k));
                }
            }
            Some(_) => return Err(self.err_invalid_bool(k)),
            None => None,
        };
        Ok(r)
    }
    pub fn bool_required(&self, k: &str) -> SynRes<bool> {
        self.bool(k)?.ok_or_else(|| self.err_required(k))
    }
    /// Reads a flag that must be omitted rather than set to false: Ok(false)
    /// if k is absent, Ok(true) if present and true, error if k = false.
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

    /// Reads k as a string literal, e.g. k = "value", None if absent.
    pub fn str(&self, k: &str) -> SynRes<Option<String>> {
        let r = match self.args.get(k) {
            Some((v, AttrParseTy::NameValue)) => match parse2::<LitStr>(v.ts2_or_err()?) {
                Ok(v) => Some(v.value()),
                _ => return Err(self.err_invalid_string(k)),
            },
            Some(_) => return Err(self.err_invalid_string(k)),
            None => None,
        };
        Ok(r)
    }
    pub fn str_required(&self, k: &str) -> SynRes<String> {
        self.str(k)?.ok_or_else(|| self.err_required(k))
    }

    /// Reads k as a nested meta list, e.g. k(a, b = c), returning the raw
    /// inner tokens as a string, None if absent.
    pub fn nested(&self, k: &str) -> SynRes<Option<String>> {
        let r = match self.args.get(k) {
            Some((v, AttrParseTy::List)) => Some(v.to_owned()),
            Some(_) => return Err(self.err_invalid_nested(k)),
            None => None,
        };
        Ok(r)
    }
    pub fn nested_required(&self, k: &str) -> SynRes<String> {
        self.nested(k)?.ok_or_else(|| self.err_required(k))
    }
    pub fn nested_into<V>(&self, k: &str) -> SynRes<Option<V>>
    where
        V: TryFrom<Self, Error = SynErr> + AttrValidate,
    {
        let r = match self.nested(k)? {
            Some(v) => Some(Self::from_ts2_into(&self.attr_debug(), k, &v.ts2_or_err()?)?),
            None => None,
        };
        Ok(r)
    }

    /// Like nested, but also accepts a bare path form: returns Some((true, k))
    /// for #[k] with no args, Some((false, tokens)) for k(...), None if absent.
    pub fn nested_with_path(&self, k: &str) -> SynRes<Option<(bool, String)>> {
        let r = match self.args.get(k) {
            Some((v, AttrParseTy::Path)) => Some((true, v.to_owned())),
            _ => self.nested(k)?.map(|v| (false, v)),
        };
        Ok(r)
    }
    pub fn nested_with_path_required(&self, k: &str) -> SynRes<(bool, String)> {
        self.nested_with_path(k)?.ok_or_else(|| self.err_required(k))
    }
    /// Like nested_with_path, converting the nested part into V, using an
    /// empty, argument-less V when k was given as a bare path.
    pub fn nested_with_path_into<V>(&self, k: &str) -> SynRes<Option<(bool, V)>>
    where
        V: TryFrom<Self, Error = SynErr> + AttrValidate,
    {
        let r = match self.nested_with_path(k)? {
            Some((path, v)) => Some((
                path,
                if path {
                    Self::init(&self.attr_debug(), k, vec![], self.span)?.try_into_with_validate()?
                } else {
                    Self::from_ts2_into(&self.attr_debug(), k, &v.ts2_or_err()?)?
                },
            )),
            None => None,
        };
        Ok(r)
    }

    /// Reads k as k = value and parses value via V::from_str, None if absent.
    pub fn parse<V>(&self, k: &str) -> SynRes<Option<V>>
    where
        V: FromStr,
    {
        let r = match self.args.get(k) {
            Some((v, AttrParseTy::NameValue)) => {
                if let Ok(v) = v.parse::<V>() {
                    Some(v)
                } else {
                    let t = type_name::<V>();
                    let msg = format!("cannot parse {v} as {t}");
                    return Err(self.err_by_key(k, &msg));
                }
            }
            Some(_) => {
                let msg = format!("should be {k} = some_value");
                return Err(self.err_by_key(k, &msg));
            }
            None => None,
        };
        Ok(r)
    }
    pub fn parse_required<V>(&self, k: &str) -> SynRes<V>
    where
        V: FromStr,
    {
        self.parse(k)?.ok_or_else(|| self.err_required(k))
    }

    // ---------------------------------------------------------------------------
    // Field and raw value accessors
    // ---------------------------------------------------------------------------

    fn field(&self) -> SynRes<(String, Attribute, Field)> {
        self.field.clone().ok_or_else(|| {
            let msg = "field: None (programmer error)";
            SynErr::new(self.span, msg)
        })
    }
    /// Model name the source field belongs to, error if built without a field.
    pub fn field_model(&self) -> SynRes<String> {
        Ok(self.field()?.0)
    }
    /// The raw attribute syntax tree on the source field, error if built
    /// without a field.
    pub fn field_attr(&self) -> SynRes<Attribute> {
        Ok(self.field()?.1)
    }
    /// Name of the source field, error if built without a field.
    pub fn field_name(&self) -> SynRes<String> {
        Ok(self.field()?.2.ident.to_token_stream().to_string())
    }
    /// Type of the source field as a string, error if built without a field.
    pub fn field_ty(&self) -> SynRes<String> {
        Ok(self.field()?.2.ty.to_token_stream().to_string())
    }

    /// Raw token string for a raw-parsed attribute (see Attr::from_field),
    /// error if this attribute was not raw-parsed.
    pub fn raw(&self) -> SynRes<String> {
        self.raw.clone().ok_or_else(|| {
            let msg = "raw: None (programmer error)";
            SynErr::new(self.span, msg)
        })
    }

    // ---------------------------------------------------------------------------
    // Validated conversion and error builders
    // ---------------------------------------------------------------------------

    /// Converts self into V, first checking that every argument key is one
    /// of V::attr_fields, erroring on any unrecognized key.
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
        let msg = format!("should be {k} for true, or {k} = false for false");
        self.err_by_key(k, &msg)
    }
    pub fn err_invalid_string(&self, k: &str) -> SynErr {
        let msg = format!(r#"should be {k} = "some_value" for string"#);
        self.err_by_key(k, &msg)
    }
    pub fn err_invalid_nested(&self, k: &str) -> SynErr {
        let msg = format!("should be {k}(some_value) for nested");
        self.err_by_key(k, &msg)
    }
    pub fn err_by_key(&self, k: &str, msg: &str) -> SynErr {
        let msg = format!("key {k} {msg}");
        self.syn_err(&msg)
    }
}

// ---------------------------------------------------------------------------
// Debug context and validation trait implementations
// ---------------------------------------------------------------------------

impl AttrDebug for Attr {
    fn attr_debug(&self) -> String {
        let Self {
            attr,
            debug,
            ..
        } = &self;
        if debug.is_empty() {
            format!("macro {attr}:")
        } else {
            format!("{debug} attr {attr}:")
        }
    }
    fn span(&self) -> Span {
        self.span
    }
}

/// Declares the set of valid argument keys for a type converted from Attr,
/// used by Attr::try_into_with_validate to reject unknown keys.
pub trait AttrValidate {
    fn attr_fields(a: &Attr) -> Vec<String>;
}
