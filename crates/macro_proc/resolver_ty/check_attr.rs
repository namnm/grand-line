use crate::prelude::*;

/// One parsed check entry, a guard method called on ctx before the resolver
/// body runs, e.g. check = authenticated or check(authz("org")).
#[derive(Clone)]
pub struct CheckAttr {
    /// Name of the ctx guard method, it should return Res of anything.
    pub name: Ident,
    /// Arguments forwarded to the guard call, empty when written as a bare name.
    pub args: Punctuated<Expr, Comma>,
}

impl CheckAttr {
    /// Parses every check entry declared under key k, accepting both the single
    /// form k = guard and the list form k(guard, other_guard(arg)).
    pub fn parse(a: &Attr, k: &str) -> SynRes<Vec<Self>> {
        let Some(exprs) = a.exprs(k)? else {
            return Ok(vec![]);
        };
        exprs.into_iter().map(|e| Self::from_expr(a, k, e)).collect()
    }

    /// The only tokens a check emits, a plain call into the ctx guard trait so
    /// the actual logic stays in that impl instead of being generated here.
    pub fn call(&self) -> Ts2 {
        let Self {
            name,
            args,
        } = self;
        quote!(ctx.#name(#args).await?;)
    }

    fn from_expr(a: &Attr, k: &str, e: Expr) -> SynRes<Self> {
        let (path, args) = match e {
            Expr::Path(e) => (e.path, Punctuated::new()),
            Expr::Call(e) => match *e.func {
                Expr::Path(f) => (f.path, e.args),
                _ => return Err(Self::err(a, k)),
            },
            _ => return Err(Self::err(a, k)),
        };

        let Some(name) = path.get_ident().cloned() else {
            return Err(Self::err(a, k));
        };
        let s = name.to_string();
        if s != s.to_snake_case() {
            let msg = format!("guard {s} is not snake case");
            return Err(a.err_by_key(k, &msg));
        }

        Ok(Self {
            name,
            args,
        })
    }

    fn err(a: &Attr, k: &str) -> SynErr {
        let msg = r#"should be a ctx method, e.g. authenticate or authz("org")"#;
        a.err_by_key(k, msg)
    }
}
