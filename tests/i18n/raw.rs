#![allow(clippy::literal_string_with_formatting_args)]

pub use grand_line::prelude::*;

#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn raw_string_value() -> Res<()> {
    let v = hashmap! {
        "name" => IntlValue::Str("Alice".into()),
    };
    intl_assert("{name}", &v, "Alice")
}

#[tokio::test]
async fn raw_int_value() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Int(42),
    };
    intl_assert("{n}", &v, "42")
}

#[tokio::test]
async fn raw_negative_int() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Int(-7),
    };
    intl_assert("{n}", &v, "-7")
}

#[tokio::test]
async fn raw_float_value() -> Res<()> {
    let v = hashmap! {
        "v" => IntlValue::Float(3.14),
    };
    intl_assert("{v}", &v, "3.14")
}

#[tokio::test]
async fn raw_at_start_of_template() -> Res<()> {
    let v = hashmap! {
        "who" => IntlValue::Str("Bob".into()),
    };
    intl_assert("{who} is here", &v, "Bob is here")
}

#[tokio::test]
async fn raw_at_end_of_template() -> Res<()> {
    let v = hashmap! {
        "who" => IntlValue::Str("Carol".into()),
    };
    intl_assert("Hello, {who}", &v, "Hello, Carol")
}

#[tokio::test]
async fn raw_only_placeholder() -> Res<()> {
    let v = hashmap! {
        "x" => IntlValue::Str("yes".into()),
    };
    intl_assert("{x}", &v, "yes")
}

#[tokio::test]
async fn raw_multiple_occurrences_of_same_var() -> Res<()> {
    let v = hashmap! {
        "a" => IntlValue::Str("x".into()),
    };
    intl_assert("{a} and {a}", &v, "x and x")
}

#[tokio::test]
async fn raw_two_adjacent_placeholders() -> Res<()> {
    let v = hashmap! {
        "a" => IntlValue::Str("foo".into()),
        "b" => IntlValue::Str("bar".into()),
    };
    intl_assert("{a}{b}", &v, "foobar")
}

#[tokio::test]
async fn raw_empty_string_value() -> Res<()> {
    let v = hashmap! {
        "x" => IntlValue::Str(String::new()),
    };
    intl_assert("before {x} after", &v, "before  after")
}

#[tokio::test]
async fn raw_int_zero() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Int(0),
    };
    intl_assert("{n}", &v, "0")
}

#[tokio::test]
async fn raw_missing_var_preserved() -> Res<()> {
    let v = hashmap! {};
    intl_assert_lenient("{name}", &v, "{name}")
}

#[tokio::test]
async fn raw_multibyte_literal_text_preserved() -> Res<()> {
    // Regression test: literal text outside placeholders must be copied as whole
    // UTF-8 chars, not byte by byte, or multi-byte chars like these get mangled.
    // "Z\u{00fc}rich caf\u{00e9}" = "Zurich cafe" with an umlaut and an accent.
    let v = hashmap! {
        "who" => IntlValue::Str("Olivia".into()),
    };
    let tpl = "Z\u{00fc}rich briefing with {who} at the caf\u{00e9}";
    let e = "Z\u{00fc}rich briefing with Olivia at the caf\u{00e9}";
    intl_assert(tpl, &v, e)
}
