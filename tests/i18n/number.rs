#![allow(clippy::literal_string_with_formatting_args)]

pub use grand_line::prelude::*;

#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// Basic formatting
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_thousands_en() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Int(1_234_567),
    };
    intl_assert("{n, number}", &v, "1,234,567")
}

#[tokio::test]
async fn number_decimal_en() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Float(1_234.56),
    };
    intl_assert("{n, number}", &v, "1,234.56")
}

#[tokio::test]
async fn number_zero_en() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Int(0),
    };
    intl_assert("{n, number}", &v, "0")
}

// ---------------------------------------------------------------------------
// Negative numbers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_negative_int() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Int(-1_234),
    };
    intl_assert("{n, number}", &v, "-1,234")
}

#[tokio::test]
async fn number_negative_float() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Float(-1_234.56),
    };
    intl_assert("{n, number}", &v, "-1,234.56")
}

// ---------------------------------------------------------------------------
// Large numbers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_billion() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Int(1_000_000_000),
    };
    intl_assert("{n, number}", &v, "1,000,000,000")
}

#[tokio::test]
async fn number_hundred() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Int(100),
    };
    intl_assert("{n, number}", &v, "100")
}

// ---------------------------------------------------------------------------
// Small decimals
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_half() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Float(0.5),
    };
    intl_assert("{n, number}", &v, "0.5")
}

#[tokio::test]
async fn number_two_cents() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Float(0.02),
    };
    intl_assert("{n, number}", &v, "0.02")
}

// ---------------------------------------------------------------------------
// Float with integer value formatted the same as Int
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_float_integer_same_as_int() -> Res<()> {
    let ctx = ctx("en")?;
    let v = hashmap! {
        "n" => IntlValue::Int(1_000),
    };
    let ri = intl("{n, number}", &v, &ctx)?;
    let v = hashmap! {
        "n" => IntlValue::Float(1_000.0),
    };
    let rf = intl("{n, number}", &v, &ctx)?;
    pretty_eq!(
        ri,
        rf,
        "float value with no fraction should format the same as the equivalent int",
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Missing variable -> placeholder preserved
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_missing_var_preserved() -> Res<()> {
    let v = hashmap! {};
    intl_assert_lenient("{n, number}", &v, "{n, number}")
}

// ---------------------------------------------------------------------------
// Wrong value type (a string where a number is required) returns Err
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_str_value_returns_err() -> Res<()> {
    let v = hashmap! {
        "n" => IntlValue::Str("not a number".into()),
    };
    let ctx = ctx("en")?;
    let r = intl("{n, number}", &v, &ctx);
    pretty_eq!(
        r.is_err(),
        true,
        "a string value for a number placeholder should return an error",
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Surrounding text
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_in_sentence() -> Res<()> {
    let v = hashmap! {
        "price" => IntlValue::Float(9_999.99),
    };
    intl_assert("Total: {price, number} USD", &v, "Total: 9,999.99 USD")
}
