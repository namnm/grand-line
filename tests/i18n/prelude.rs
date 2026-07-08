#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

pub use grand_line::prelude::*;

// To regenerate the test blob (run from project root):
//   cargo install icu4x-datagen --version "^2.2"
//   icu4x-datagen --markers all --locales en --format blob2 \
//     --out tests/i18n/dist-icu4x-testdata-en/locales.postcard
static BLOB: &[u8] = include_bytes!("dist-icu4x-testdata-en/locales.postcard");

// ---------------------------------------------------------------------------
// Timestamps (milliseconds since Unix epoch, UTC)
// ---------------------------------------------------------------------------

/// 2024-01-15 00:00:00 UTC.
pub const JAN_15_2024: i64 = 1_705_276_800_000;
/// Same timestamp as `JAN_15_2024` but as f64 (exactly representable, under 2^53).
pub const JAN_15_2024_F64: f64 = 1_705_276_800_000.0;
/// 2024-01-15 14:30:00 UTC.
pub const JAN_15_2024_1430: i64 = 1_705_329_000_000;
/// 1970-01-01 00:00:00 UTC  (Unix epoch zero).
pub const EPOCH_ZERO: i64 = 0;
/// 2024-12-31 23:59:59 UTC.
pub const DEC_31_2024_EOD: i64 = 1_735_689_599_000;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run format_template with English locale and a fixed key->value lookup.
pub fn intl(tpl: &str, v: &[(&str, IntlValue)], e: &str) -> Res<()> {
    let ctx = IntlFormatter::init(BLOB, "en")?;
    let r = format_template(
        tpl,
        &|name| v.iter().find(|(k, _)| *k == name).map(|(_, v)| v.clone()),
        &ctx,
    );
    pretty_eq!(r, e);
    Ok(())
}

/// Build an IntlFormatter for the given locale.
pub fn ctx(locale: &str) -> Res<IntlFormatter> {
    IntlFormatter::init(BLOB, locale)
}

/// Run format_template with a closure that always returns None (all vars missing).
pub fn intl_no_vars(tpl: &str, e: &str) -> Res<()> {
    let ctx = IntlFormatter::init(BLOB, "en")?;
    let r = format_template(tpl, &|_| None, &ctx);
    pretty_eq!(r, e);
    Ok(())
}
