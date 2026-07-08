pub use grand_line::prelude::*;

#[path = "./prelude.rs"]
mod prelude;
use prelude::*;

// Plain `{name}` substitution - no type annotation, just a toString.

#[tokio::test]
async fn raw_string_value() -> Res<()> {
    intl("{name}", &[("name", IntlValue::Str("Alice".into()))], "Alice")
}

#[tokio::test]
async fn raw_int_value() -> Res<()> {
    // No number formatting - just the integer's string representation.
    intl("{n}", &[("n", IntlValue::Int(42))], "42")
}

#[tokio::test]
async fn raw_negative_int() -> Res<()> {
    intl("{n}", &[("n", IntlValue::Int(-7))], "-7")
}

#[tokio::test]
async fn raw_float_value() -> Res<()> {
    intl("{v}", &[("v", IntlValue::Float(3.14))], "3.14")
}

#[tokio::test]
async fn raw_at_start_of_template() -> Res<()> {
    intl("{who} is here", &[("who", IntlValue::Str("Bob".into()))], "Bob is here")
}

#[tokio::test]
async fn raw_at_end_of_template() -> Res<()> {
    intl(
        "Hello, {who}",
        &[("who", IntlValue::Str("Carol".into()))],
        "Hello, Carol",
    )
}

#[tokio::test]
async fn raw_only_placeholder() -> Res<()> {
    intl("{x}", &[("x", IntlValue::Str("yes".into()))], "yes")
}

#[tokio::test]
async fn raw_multiple_occurrences_of_same_var() -> Res<()> {
    // Every occurrence is independently substituted via lookup.
    intl("{a} and {a}", &[("a", IntlValue::Str("x".into()))], "x and x")
}

#[tokio::test]
async fn raw_two_adjacent_placeholders() -> Res<()> {
    let v = [("a", IntlValue::Str("foo".into())), ("b", IntlValue::Str("bar".into()))];
    intl("{a}{b}", &v, "foobar")
}

#[tokio::test]
async fn raw_empty_string_value() -> Res<()> {
    intl(
        "before {x} after",
        &[("x", IntlValue::Str(String::new()))],
        "before  after",
    )
}

#[tokio::test]
async fn raw_int_zero() -> Res<()> {
    intl("{n}", &[("n", IntlValue::Int(0))], "0")
}

#[tokio::test]
async fn raw_missing_var_preserved() -> Res<()> {
    intl_no_vars("{name}", "{name}")
}
