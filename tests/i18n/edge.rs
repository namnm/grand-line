pub use grand_line::prelude::*;

#[path = "./prelude.rs"]
mod prelude;
use prelude::*;

// ---------------------------------------------------------------------------
// Template structure edge cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_empty_template() -> Res<()> {
    intl("", &[], "")
}

#[tokio::test]
async fn edge_no_placeholders_is_passthrough() -> Res<()> {
    intl("just a plain string", &[], "just a plain string")
}

#[tokio::test]
async fn edge_literal_text_only() -> Res<()> {
    intl("Hello, World!", &[], "Hello, World!")
}

// ---------------------------------------------------------------------------
// Unclosed / malformed braces
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_unclosed_brace_preserved_to_end() -> Res<()> {
    // When the opening `{` has no matching `}`, the remainder (including the
    // `{`) is appended verbatim and parsing stops.
    intl_no_vars("start {unclosed", "start {unclosed")
}

#[tokio::test]
async fn edge_text_before_unclosed_brace() -> Res<()> {
    // Text before the unclosed brace is emitted normally.
    let out = {
        let ctx = ctx("en")?;
        format_template("prefix {bad", &|_| None, &ctx)
    };
    pretty_eq!(out, "prefix {bad");
    Ok(())
}

// ---------------------------------------------------------------------------
// Invalid / unusual variable names
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_space_in_var_name_preserved() -> Res<()> {
    // "not valid" contains a space -> parse_placeholder returns None -> placeholder kept.
    intl_no_vars("{not valid}", "{not valid}")
}

#[tokio::test]
async fn edge_empty_var_name_preserved() -> Res<()> {
    // `{}` is an empty var name -> parse_placeholder returns None -> kept as-is.
    intl_no_vars("{}", "{}")
}

// ---------------------------------------------------------------------------
// Unknown placeholder type falls back to raw toString
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_unknown_type_treated_as_raw() -> Res<()> {
    // "currency" is not a recognised type -> Ph::Raw -> val.to_string().
    intl("{amount, currency}", &[("amount", IntlValue::Int(100))], "100")
}

#[tokio::test]
async fn edge_unknown_type_with_tail_treated_as_raw() -> Res<()> {
    // Type "money" with extra args - still falls through to raw.
    intl("{v, money, USD}", &[("v", IntlValue::Float(9.99))], "9.99")
}

// ---------------------------------------------------------------------------
// Whitespace in placeholder is trimmed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_spaces_around_var_name_trimmed() -> Res<()> {
    // `{ name }` -> cut() trims -> same as `{name}`.
    intl("{ name }", &[("name", IntlValue::Str("Alice".into()))], "Alice")
}

#[tokio::test]
async fn edge_spaces_around_type_trimmed() -> Res<()> {
    intl("{ n , number }", &[("n", IntlValue::Int(1_000))], "1,000")
}

// ---------------------------------------------------------------------------
// Template with only whitespace
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_whitespace_only_template() -> Res<()> {
    intl("   ", &[], "   ")
}

// ---------------------------------------------------------------------------
// Multiple placeholders, some missing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_partial_var_resolution() -> Res<()> {
    // Only "a" is supplied, "b" and "c" placeholders are preserved.
    let v = [("a", IntlValue::Str("HERE".into()))];
    intl("{a} {b} {c}", &v, "HERE {b} {c}")
}

// ---------------------------------------------------------------------------
// Numeric var name (all digits are alphanumeric, so it IS a valid var name)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_numeric_var_name_is_valid() -> Res<()> {
    // "0" is alphanumeric -> valid var name -> substituted when provided.
    intl("{0}", &[("0", IntlValue::Str("zero".into()))], "zero")
}

#[tokio::test]
async fn edge_numeric_var_name_missing_preserved() -> Res<()> {
    intl_no_vars("{42}", "{42}")
}

// ---------------------------------------------------------------------------
// Nested braces inside a plural case body (depth tracking)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_plural_with_inner_braces_in_case() -> Res<()> {
    // The `{` in "section {A}" is inside a plural case body - depth should be
    // tracked correctly so `}` there doesn't terminate the outer `{c, plural, ...}`.
    //
    // BUT: find_case sees "section {A}" as content and returns it as-is.
    // `#` is replaced with the count. The inner `{A}` is NOT a placeholder here --
    // format_template only runs on the top-level result, not on case bodies.
    let tpl = "{c, plural, one{section {A}} other{# sections}}";
    let v = [("c", IntlValue::Int(1))];
    intl(tpl, &v, "section {A}")
}
