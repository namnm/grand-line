pub use grand_line::prelude::*;

#[path = "./prelude.rs"]
mod prelude;
use prelude::*;

// ---------------------------------------------------------------------------
// Date styles
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_medium_en() -> Res<()> {
    intl("{d, date}", &[("d", IntlValue::Int(JAN_15_2024))], "Jan 15, 2024")
}

#[tokio::test]
async fn date_short_en() -> Res<()> {
    intl("{d, date, short}", &[("d", IntlValue::Int(JAN_15_2024))], "1/15/24")
}

#[tokio::test]
async fn date_long_en() -> Res<()> {
    intl(
        "{d, date, long}",
        &[("d", IntlValue::Int(JAN_15_2024))],
        "January 15, 2024",
    )
}

#[tokio::test]
async fn date_full_en() -> Res<()> {
    intl(
        "{d, date, full}",
        &[("d", IntlValue::Int(JAN_15_2024))],
        "January 15, 2024",
    )
}

#[tokio::test]
async fn date_unknown_style_falls_to_medium() -> Res<()> {
    // Any unrecognised style key maps to the "date" (medium) formatter.
    intl(
        "{d, date, whatever}",
        &[("d", IntlValue::Int(JAN_15_2024))],
        "Jan 15, 2024",
    )
}

// ---------------------------------------------------------------------------
// Time style
// ---------------------------------------------------------------------------

#[tokio::test]
async fn time_en() -> Res<()> {
    // ICU4X uses U+202F narrow no-break space before AM/PM per CLDR spec.
    let tpl = "{t, time}";
    let v = [("t", IntlValue::Int(JAN_15_2024_1430))];
    let e = "2:30:00\u{202f}PM";
    intl(tpl, &v, e)
}

#[tokio::test]
async fn time_midnight_en() -> Res<()> {
    // 2024-01-15 00:00:00 UTC -> 12:00 AM
    let v = [("t", IntlValue::Int(JAN_15_2024))];
    intl("{t, time}", &v, "12:00:00\u{202f}AM")
}

// ---------------------------------------------------------------------------
// Epoch zero
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_epoch_zero_medium() -> Res<()> {
    intl("{d, date}", &[("d", IntlValue::Int(EPOCH_ZERO))], "Jan 1, 1970")
}

#[tokio::test]
async fn date_epoch_zero_long() -> Res<()> {
    intl(
        "{d, date, long}",
        &[("d", IntlValue::Int(EPOCH_ZERO))],
        "January 1, 1970",
    )
}

#[tokio::test]
async fn date_epoch_zero_short() -> Res<()> {
    intl("{d, date, short}", &[("d", IntlValue::Int(EPOCH_ZERO))], "1/1/70")
}

// ---------------------------------------------------------------------------
// End of year (Dec 31)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_dec_31_medium() -> Res<()> {
    intl("{d, date}", &[("d", IntlValue::Int(DEC_31_2024_EOD))], "Dec 31, 2024")
}

// ---------------------------------------------------------------------------
// Float timestamp is treated as integer milliseconds
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_float_ts_same_as_int() -> Res<()> {
    // IntlValue::Float uses as_int() -> truncation to i64, same result as Int.
    let v_int = [("d", IntlValue::Int(JAN_15_2024))];
    let v_flt = [("d", IntlValue::Float(JAN_15_2024_F64))];
    let ctx = ctx("en")?;
    let r_int = format_template(
        "{d, date}",
        &|n| v_int.iter().find(|(k, _)| *k == n).map(|(_, v)| v.clone()),
        &ctx,
    );
    let r_flt = format_template(
        "{d, date}",
        &|n| v_flt.iter().find(|(k, _)| *k == n).map(|(_, v)| v.clone()),
        &ctx,
    );
    pretty_eq!(r_int, r_flt);
    Ok(())
}

// ---------------------------------------------------------------------------
// Missing variable -> placeholder preserved verbatim
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_missing_var_preserved() -> Res<()> {
    intl_no_vars("{d, date}", "{d, date}")
}

#[tokio::test]
async fn date_missing_var_short_preserved() -> Res<()> {
    intl_no_vars("{d, date, short}", "{d, date, short}")
}

#[tokio::test]
async fn time_missing_var_preserved() -> Res<()> {
    intl_no_vars("{t, time}", "{t, time}")
}

// ---------------------------------------------------------------------------
// Surrounding text is preserved alongside the formatted date
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_in_sentence() -> Res<()> {
    let v = [("d", IntlValue::Int(JAN_15_2024))];
    intl("Joined on {d, date, long}.", &v, "Joined on January 15, 2024.")
}
