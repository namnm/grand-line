use crate::prelude::*;
use chrono::{Datelike as _, TimeZone as _, Timelike as _, Utc};
use fixed_decimal::Decimal;
use icu_calendar::Date;
use icu_datetime::{DateTimeFormatter, NoCalendarFormatter, fieldsets, input::Time as IcuTime};
use icu_decimal::DecimalFormatter;
use icu_locale_core::Locale;
use icu_plurals::{PluralCategory, PluralRules};
use icu_provider_blob::BlobDataProvider;

// ---------------------------------------------------------------------------
// Context -- owned formatting callbacks, locale + i18n captured inside
// ---------------------------------------------------------------------------

pub struct IntlFormatter {
    pub date: Box<dyn Fn(i64, &str) -> String>,
    pub number: Box<dyn Fn(f64) -> String>,
    /// Returns a CLDR plural category: "zero"|"one"|"two"|"few"|"many"|"other".
    pub plural: Box<dyn Fn(i64) -> &'static str>,
}

// ---------------------------------------------------------------------------
// ICU4X constructor (feature-gated)
// ---------------------------------------------------------------------------

impl IntlFormatter {
    /// Build an `IntlFormatter` backed by a real ICU4X blob provider for the given locale.
    ///
    /// All formatters are initialised eagerly, errors during init are returned
    /// as Err so callers can handle missing locale data at startup, not at
    /// format time.
    pub fn init(blob: &'static [u8], locale_str: &str) -> Res<Self> {
        let provider = BlobDataProvider::try_new_from_static_blob(blob).map_err(|e| MyErr::IcuBlob(e.to_string()))?;

        let locale: Locale = locale_str
            .parse()
            .map_err(|_| MyErr::InvalidLocale(locale_str.to_owned()))?;

        let prefs = (&locale).into();

        let fmt_medium = DateTimeFormatter::try_new_with_buffer_provider(&provider, prefs, fieldsets::YMD::medium())
            .map_err(|e| MyErr::IcuInit(e.to_string()))?;

        let fmt_short =
            DateTimeFormatter::try_new_with_buffer_provider(&provider, (&locale).into(), fieldsets::YMD::short())
                .map_err(|e| MyErr::IcuInit(e.to_string()))?;

        let fmt_long =
            DateTimeFormatter::try_new_with_buffer_provider(&provider, (&locale).into(), fieldsets::YMD::long())
                .map_err(|e| MyErr::IcuInit(e.to_string()))?;

        let fmt_time =
            NoCalendarFormatter::try_new_with_buffer_provider(&provider, (&locale).into(), fieldsets::T::short())
                .map_err(|e| MyErr::IcuInit(e.to_string()))?;

        let number_fmt =
            DecimalFormatter::try_new_with_buffer_provider(&provider, (&locale).into(), Default::default())
                .map_err(|e| MyErr::IcuInit(e.to_string()))?;

        let plural_rules = PluralRules::try_new_cardinal_with_buffer_provider(&provider, (&locale).into())
            .map_err(|e| MyErr::IcuInit(e.to_string()))?;

        let r = Self {
            date: Box::new(move |ts_ms, style| {
                let Some(dt) = Utc.timestamp_opt(ts_ms / 1000, 0).single() else {
                    return String::new();
                };
                if style == "time" {
                    let t = IcuTime::try_new(dt.hour() as u8, dt.minute() as u8, dt.second() as u8, 0);
                    let Ok(t) = t else {
                        return String::new();
                    };
                    return fmt_time.format(&t).to_string();
                }
                let d = Date::try_new_gregorian(dt.year(), dt.month() as u8, dt.day() as u8);
                let Ok(d) = d else {
                    return String::new();
                };
                let fmt = match style {
                    "short_date" => &fmt_short,
                    "long_date" => &fmt_long,
                    _ => &fmt_medium,
                };
                fmt.format(&d).to_string()
            }),

            number: Box::new(move |n| {
                let Ok(fd) = Decimal::from_str(&n.to_string()) else {
                    return n.to_string();
                };
                number_fmt.format_to_string(&fd)
            }),

            plural: Box::new(move |count| match plural_rules.category_for(count.unsigned_abs()) {
                PluralCategory::Zero => "zero",
                PluralCategory::One => "one",
                PluralCategory::Two => "two",
                PluralCategory::Few => "few",
                PluralCategory::Many => "many",
                _ => "other",
            }),
        };

        Ok(r)
    }
}
