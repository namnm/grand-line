pub use grand_line::prelude::*;

#[path = "./prelude.rs"]
mod prelude;
use prelude::*;

// ---------------------------------------------------------------------------
// Basic formatting
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_thousands_en() -> Res<()> {
    intl("{n, number}", &[("n", IntlValue::Int(1_234_567))], "1,234,567")
}

#[tokio::test]
async fn number_decimal_en() -> Res<()> {
    intl("{n, number}", &[("n", IntlValue::Float(1_234.56))], "1,234.56")
}

#[tokio::test]
async fn number_zero_en() -> Res<()> {
    intl("{n, number}", &[("n", IntlValue::Int(0))], "0")
}

// ---------------------------------------------------------------------------
// Negative numbers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_negative_int() -> Res<()> {
    intl("{n, number}", &[("n", IntlValue::Int(-1_234))], "-1,234")
}

#[tokio::test]
async fn number_negative_float() -> Res<()> {
    intl("{n, number}", &[("n", IntlValue::Float(-1_234.56))], "-1,234.56")
}

// ---------------------------------------------------------------------------
// Large numbers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_billion() -> Res<()> {
    intl("{n, number}", &[("n", IntlValue::Int(1_000_000_000))], "1,000,000,000")
}

#[tokio::test]
async fn number_hundred() -> Res<()> {
    intl("{n, number}", &[("n", IntlValue::Int(100))], "100")
}

// ---------------------------------------------------------------------------
// Small decimals
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_half() -> Res<()> {
    intl("{n, number}", &[("n", IntlValue::Float(0.5))], "0.5")
}

#[tokio::test]
async fn number_two_cents() -> Res<()> {
    intl("{n, number}", &[("n", IntlValue::Float(0.02))], "0.02")
}

// ---------------------------------------------------------------------------
// Float with integer value formatted the same as Int
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_float_integer_same_as_int() -> Res<()> {
    let ctx = ctx("en")?;
    let ri = format_template("{n, number}", &|_| Some(IntlValue::Int(1_000)), &ctx);
    let rf = format_template("{n, number}", &|_| Some(IntlValue::Float(1_000.0)), &ctx);
    pretty_eq!(ri, rf);
    Ok(())
}

// ---------------------------------------------------------------------------
// Missing variable -> placeholder preserved
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_missing_var_preserved() -> Res<()> {
    intl_no_vars("{n, number}", "{n, number}")
}

// ---------------------------------------------------------------------------
// Surrounding text
// ---------------------------------------------------------------------------

#[tokio::test]
async fn number_in_sentence() -> Res<()> {
    let v = [("price", IntlValue::Float(9_999.99))];
    intl("Total: {price, number} USD", &v, "Total: 9,999.99 USD")
}
