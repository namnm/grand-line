#![allow(clippy::literal_string_with_formatting_args)]

pub use grand_line::prelude::*;

#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// Date styles
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_medium_en() -> Res<()> {
    let v = hashmap! {
        "d" => IntlValue::Int(JAN_15_2024),
    };
    intl_assert("{d, date}", &v, "Jan 15, 2024")
}

#[tokio::test]
async fn date_short_en() -> Res<()> {
    let v = hashmap! {
        "d" => IntlValue::Int(JAN_15_2024),
    };
    intl_assert("{d, date, short}", &v, "1/15/24")
}

#[tokio::test]
async fn date_long_en() -> Res<()> {
    let v = hashmap! {
        "d" => IntlValue::Int(JAN_15_2024),
    };
    intl_assert("{d, date, long}", &v, "January 15, 2024")
}

#[tokio::test]
async fn date_full_en() -> Res<()> {
    let v = hashmap! {
        "d" => IntlValue::Int(JAN_15_2024),
    };
    intl_assert("{d, date, full}", &v, "January 15, 2024")
}

#[tokio::test]
async fn date_unknown_style_falls_to_medium() -> Res<()> {
    // Any unrecognised style key maps to the "date" (medium) formatter.
    let v = hashmap! {
        "d" => IntlValue::Int(JAN_15_2024),
    };
    intl_assert("{d, date, whatever}", &v, "Jan 15, 2024")
}

// ---------------------------------------------------------------------------
// Time style
// ---------------------------------------------------------------------------

#[tokio::test]
async fn time_en() -> Res<()> {
    // ICU4X uses U+202F narrow no-break space before AM/PM per CLDR spec.
    let tpl = "{t, time}";
    let v = hashmap! {
        "t" => IntlValue::Int(JAN_15_2024_1430),
    };
    let e = "2:30:00\u{202f}PM";
    intl_assert(tpl, &v, e)
}

#[tokio::test]
async fn time_midnight_en() -> Res<()> {
    // 2024-01-15 00:00:00 UTC -> 12:00 AM
    let v = hashmap! {
        "t" => IntlValue::Int(JAN_15_2024),
    };
    intl_assert("{t, time}", &v, "12:00:00\u{202f}AM")
}

// ---------------------------------------------------------------------------
// Epoch zero
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_epoch_zero_medium() -> Res<()> {
    let v = hashmap! {
        "d" => IntlValue::Int(EPOCH_ZERO),
    };
    intl_assert("{d, date}", &v, "Jan 1, 1970")
}

#[tokio::test]
async fn date_epoch_zero_long() -> Res<()> {
    let v = hashmap! {
        "d" => IntlValue::Int(EPOCH_ZERO),
    };
    intl_assert("{d, date, long}", &v, "January 1, 1970")
}

#[tokio::test]
async fn date_epoch_zero_short() -> Res<()> {
    let v = hashmap! {
        "d" => IntlValue::Int(EPOCH_ZERO),
    };
    intl_assert("{d, date, short}", &v, "1/1/70")
}

// ---------------------------------------------------------------------------
// Sub-second negative timestamp floors to the previous day, not epoch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_before_epoch_sub_second_floors_to_previous_day() -> Res<()> {
    // -500ms is 1969-12-31 23:59:59.500 UTC. Truncating division (-500 / 1000 == 0)
    // would wrongly round this up to the epoch itself, Jan 1 1970.
    let v = hashmap! {
        "d" => IntlValue::Int(-500),
    };
    intl_assert("{d, date}", &v, "Dec 31, 1969")
}

// ---------------------------------------------------------------------------
// End of year (Dec 31)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_dec_31_medium() -> Res<()> {
    let v = hashmap! {
        "d" => IntlValue::Int(DEC_31_2024_EOD),
    };
    intl_assert("{d, date}", &v, "Dec 31, 2024")
}

// ---------------------------------------------------------------------------
// Float timestamp is treated as integer milliseconds
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_float_ts_same_as_int() -> Res<()> {
    // IntlValue::Float uses as_int() -> truncation to i64, same result as Int.
    let ctx = ctx("en")?;
    let v = hashmap! {
        "d" => IntlValue::Int(JAN_15_2024),
    };
    let ri = intl("{d, date}", &v, &ctx)?;
    let v = hashmap! {
        "d" => IntlValue::Float(JAN_15_2024_F64),
    };
    let rf = intl("{d, date}", &v, &ctx)?;
    pretty_eq!(
        ri,
        rf,
        "float timestamp should format the same as the truncated int timestamp",
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Timestamp outside chrono's representable range
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_extreme_timestamp_returns_err() -> Res<()> {
    // i64::MAX milliseconds is far outside the range Utc::timestamp_opt can represent.
    let v = hashmap! {
        "d" => IntlValue::Int(i64::MAX),
    };
    let ctx = ctx("en")?;
    let r = intl("{d, date}", &v, &ctx);
    pretty_eq!(
        r.is_err(),
        true,
        "extreme timestamp should return an error instead of formatting",
    );
    Ok(())
}

#[tokio::test]
async fn time_extreme_timestamp_returns_err() -> Res<()> {
    let v = hashmap! {
        "t" => IntlValue::Int(i64::MAX),
    };
    let ctx = ctx("en")?;
    let r = intl("{t, time}", &v, &ctx);
    pretty_eq!(
        r.is_err(),
        true,
        "extreme timestamp should return an error instead of formatting",
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Wrong value type (a string where a number is required) returns Err
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_str_value_returns_err() -> Res<()> {
    let v = hashmap! {
        "d" => IntlValue::Str("not a timestamp".into()),
    };
    let ctx = ctx("en")?;
    let r = intl("{d, date}", &v, &ctx);
    pretty_eq!(
        r.is_err(),
        true,
        "a string value for a date placeholder should return an error",
    );
    Ok(())
}

#[tokio::test]
async fn time_str_value_returns_err() -> Res<()> {
    let v = hashmap! {
        "t" => IntlValue::Str("not a timestamp".into()),
    };
    let ctx = ctx("en")?;
    let r = intl("{t, time}", &v, &ctx);
    pretty_eq!(
        r.is_err(),
        true,
        "a string value for a time placeholder should return an error",
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Missing variable returns Err by default, preserved verbatim when allowed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_missing_var_returns_err() -> Res<()> {
    let v = hashmap! {};
    let ctx = ctx("en")?;
    let r = intl("{d, date}", &v, &ctx);
    pretty_eq!(r.is_err(), true, "missing var should return an error by default");
    Ok(())
}

#[tokio::test]
async fn date_missing_var_preserved() -> Res<()> {
    let v = hashmap! {};
    intl_assert_lenient("{d, date}", &v, "{d, date}")
}

#[tokio::test]
async fn date_missing_var_short_preserved() -> Res<()> {
    let v = hashmap! {};
    intl_assert_lenient("{d, date, short}", &v, "{d, date, short}")
}

#[tokio::test]
async fn time_missing_var_preserved() -> Res<()> {
    let v = hashmap! {};
    intl_assert_lenient("{t, time}", &v, "{t, time}")
}

// ---------------------------------------------------------------------------
// Surrounding text is preserved alongside the formatted date
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_in_sentence() -> Res<()> {
    let v = hashmap! {
        "d" => IntlValue::Int(JAN_15_2024),
    };
    intl_assert("Joined on {d, date, long}.", &v, "Joined on January 15, 2024.")
}
