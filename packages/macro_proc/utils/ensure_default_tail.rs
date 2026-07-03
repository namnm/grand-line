use crate::prelude::*;

/// If body (a sequence of statements, no surrounding braces) doesn't end in a tail
/// expression, append Default::default() so a resolver that doesn't need to add
/// anything extra can leave the body empty (or side-effect-only).
pub fn ensure_default_tail(body: Ts2) -> SynRes<Ts2> {
    let stmts = Block::parse_within.parse2(body)?;
    let has_tail = matches!(stmts.last(), Some(Stmt::Expr(_, None)));
    let default_tail = (!has_tail).then(|| quote!(Default::default()));

    let r = quote! {{
        #(#stmts)*
        #default_tail
    }};
    Ok(r)
}
