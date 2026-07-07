use crate::prelude::*;

/// Same as unwrap_option_str, but parses the inner type string into tokens.
/// Returns (was_option, inner_type), inner_type is ty itself when not wrapped.
pub fn unwrap_option<D>(ty: D) -> SynRes<(bool, Ts2)>
where
    D: Display,
{
    let (opt, uw_str) = unwrap_option_str(ty);
    Ok((opt, uw_str.ts2_or_err()?))
}

/// Strips a surrounding Option<..> from a type's string form, ignoring spaces.
/// Returns (was_option, inner_type), inner_type is ty itself when not wrapped.
pub fn unwrap_option_str<D>(ty: D) -> (bool, String)
where
    D: Display,
{
    let uw_str = ty.to_string().replace(' ', "");
    if uw_str.starts_with("Option<") {
        return (true, uw_str[7..uw_str.len() - 1].to_owned());
    }
    (false, uw_str)
}
