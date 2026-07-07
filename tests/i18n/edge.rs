#![allow(clippy::literal_string_with_formatting_args)]

pub use grand_line::prelude::*;

#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// Template structure edge cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_empty_template() -> Res<()> {
    let v = hashmap! {};
    intl_assert("", &v, "")
}

#[tokio::test]
async fn edge_no_placeholders_is_passthrough() -> Res<()> {
    let v = hashmap! {};
    intl_assert("just a plain string", &v, "just a plain string")
}

#[tokio::test]
async fn edge_literal_text_only() -> Res<()> {
    let v = hashmap! {};
    intl_assert("Hello, World!", &v, "Hello, World!")
}

// ---------------------------------------------------------------------------
// Unclosed / malformed braces
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_unclosed_brace_preserved_to_end() -> Res<()> {
    // When the opening `{` has no matching `}`, the remainder (including the
    // `{`) is appended verbatim and parsing stops.
    let v = hashmap! {};
    intl_assert("start {unclosed", &v, "start {unclosed")
}

#[tokio::test]
async fn edge_text_before_unclosed_brace() -> Res<()> {
    // Text before the unclosed brace is emitted normally.
    let out = {
        let ctx = ctx("en")?;
        let v = hashmap! {};
        intl("prefix {bad", &v, &ctx)?
    };
    pretty_eq!(
        out,
        "prefix {bad",
        "text before an unclosed brace should be emitted verbatim",
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Invalid / unusual variable names
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_space_in_var_name_preserved() -> Res<()> {
    // "not valid" contains a space -> parse_placeholder returns None -> placeholder kept.
    let v = hashmap! {};
    intl_assert("{not valid}", &v, "{not valid}")
}

#[tokio::test]
async fn edge_empty_var_name_preserved() -> Res<()> {
    // `{}` is an empty var name -> parse_placeholder returns None -> kept as-is.
    let v = hashmap! {};
    intl_assert("{}", &v, "{}")
}

// ---------------------------------------------------------------------------
// Unknown placeholder type falls back to raw toString
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_unknown_type_treated_as_raw() -> Res<()> {
    // "currency" is not a recognised type -> Ph::Raw -> val.to_string().
    let v = hashmap! {
        "amount" => IntlValue::Int(100),
    };
    intl_assert("{amount, currency}", &v, "100")
}

#[tokio::test]
async fn edge_unknown_type_with_tail_treated_as_raw() -> Res<()> {
    // Type "money" with extra args - still falls through to raw.
    let v = hashmap! {
        "v" => IntlValue::Float(9.99),
    };
    intl_assert("{v, money, USD}", &v, "9.99")
}

// ---------------------------------------------------------------------------
// Whitespace in placeholder is trimmed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_spaces_around_var_name_trimmed() -> Res<()> {
    // `{ name }` -> cut() trims -> same as `{name}`.
    let v = hashmap! {
        "name" => IntlValue::Str("Alice".into()),
    };
    intl_assert("{ name }", &v, "Alice")
}

#[tokio::test]
async fn edge_spaces_around_type_trimmed() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Int(1_000),
    };
    intl_assert("{ n , number }", &v, "1,000")
}

// ---------------------------------------------------------------------------
// Template with only whitespace
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_whitespace_only_template() -> Res<()> {
    let v = hashmap! {};
    intl_assert("   ", &v, "   ")
}

// ---------------------------------------------------------------------------
// Multiple placeholders, some missing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_partial_var_resolution() -> Res<()> {
    // Only "a" is supplied, "b" and "c" placeholders are preserved.
    let v = hashmap! {
        "a" => IntlValue::Str("HERE".into()),
    };
    intl_assert_lenient("{a} {b} {c}", &v, "HERE {b} {c}")
}

// ---------------------------------------------------------------------------
// Numeric var name (all digits are alphanumeric, so it IS a valid var name)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn edge_numeric_var_name_is_valid() -> Res<()> {
    // "0" is alphanumeric -> valid var name -> substituted when provided.
    let v = hashmap! {
        "0" => IntlValue::Str("zero".into()),
    };
    intl_assert("{0}", &v, "zero")
}

#[tokio::test]
async fn edge_numeric_var_name_missing_preserved() -> Res<()> {
    let v = hashmap! {};
    intl_assert_lenient("{42}", &v, "{42}")
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
    // intl only runs on the top-level result, not on case bodies.
    let tpl = "{c, plural, one{section {A}} other{# sections}}";
    let v = hashmap! {
        "c" => IntlValue::Int(1),
    };
    intl_assert(tpl, &v, "section {A}")
}
