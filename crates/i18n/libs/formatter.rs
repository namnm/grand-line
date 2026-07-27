use crate::prelude::*;
use chrono::{DateTime, Datelike as _, Timelike as _, Utc};
use fixed_decimal::Decimal;
use icu_calendar::Date;
use icu_datetime::{
    DateTimeFormatter, NoCalendarFormatter,
    fieldsets::{T, YMD},
    input::Time,
};
use icu_decimal::DecimalFormatter;
use icu_locale_core::Locale;
use icu_plurals::{PluralCategory, PluralRules};
use icu_provider_blob::BlobDataProvider;

// ---------------------------------------------------------------------------
// Context -- owned formatting callbacks, locale + i18n captured inside
// ---------------------------------------------------------------------------

pub struct IntlFormatter {
    pub(crate) date: Box<dyn Fn(i64, &str) -> Res<String>>,
    pub(crate) number: Box<dyn Fn(f64) -> String>,
    /// Returns a CLDR plural category: "zero"|"one"|"two"|"few"|"many"|"other".
    pub(crate) plural: Box<dyn Fn(i64) -> &'static str>,
    /// When false (default), intl returns Err for a placeholder whose var is
    /// missing from vars. When true, the placeholder is preserved verbatim.
    pub(crate) allow_missing_vars: bool,
}

// ---------------------------------------------------------------------------
// ICU4X constructor (feature-gated)
// ---------------------------------------------------------------------------

impl IntlFormatter {
    /// Build an IntlFormatter backed by a real ICU4X blob provider for the given locale.
    ///
    /// All formatters are initialised eagerly, errors during init are returned
    /// as Err so callers can handle missing locale data at startup, not at
    /// format time.
    pub fn init(blob: &'static [u8], locale_str: &str) -> Res<Self> {
        let b = BlobDataProvider::try_new_from_static_blob(blob).map_err(|e| MyErr::IcuBlob {
            inner: e.to_string(),
        })?;

        let l: Locale = locale_str.parse().map_err(|_| MyErr::InvalidLocale {
            locale: locale_str.to_owned(),
        })?;

        let f_medium = DateTimeFormatter::try_new_with_buffer_provider(&b, (&l).into(), YMD::medium());
        let f_medium = f_medium.map_err(|e| MyErr::IcuInit {
            inner: e.to_string(),
        })?;

        let f_short = DateTimeFormatter::try_new_with_buffer_provider(&b, (&l).into(), YMD::short());
        let f_short = f_short.map_err(|e| MyErr::IcuInit {
            inner: e.to_string(),
        })?;

        let f_long = DateTimeFormatter::try_new_with_buffer_provider(&b, (&l).into(), YMD::long());
        let f_long = f_long.map_err(|e| MyErr::IcuInit {
            inner: e.to_string(),
        })?;

        let f_time = NoCalendarFormatter::try_new_with_buffer_provider(&b, (&l).into(), T::short());
        let f_time = f_time.map_err(|e| MyErr::IcuInit {
            inner: e.to_string(),
        })?;

        let f_number = DecimalFormatter::try_new_with_buffer_provider(&b, (&l).into(), Default::default());
        let f_number = f_number.map_err(|e| MyErr::IcuInit {
            inner: e.to_string(),
        })?;

        let plurals = PluralRules::try_new_cardinal_with_buffer_provider(&b, (&l).into());
        let plurals = plurals.map_err(|e| MyErr::IcuInit {
            inner: e.to_string(),
        })?;

        let r = Self {
            date: Box::new(move |ts_ms, style| {
                let err = || MyErr::InvalidTimestamp {
                    value: ts_ms,
                };

                // from_timestamp_millis floors correctly for negative ts_ms, unlike
                // manual ts_ms / 1000 which truncates toward zero and would round a
                // pre-1970 sub-second remainder the wrong way.
                let Some(dt) = DateTime::<Utc>::from_timestamp_millis(ts_ms) else {
                    return Err(err().into());
                };

                if style == "time" {
                    let t = Time::try_new(dt.hour() as u8, dt.minute() as u8, dt.second() as u8, 0);
                    let Ok(t) = t else {
                        return Err(err().into());
                    };
                    return Ok(f_time.format(&t).to_string());
                }

                let d = Date::try_new_gregorian(dt.year(), dt.month() as u8, dt.day() as u8);
                let Ok(d) = d else {
                    return Err(err().into());
                };

                let fmt = match style {
                    "short_date" => &f_short,
                    "long_date" => &f_long,
                    _ => &f_medium,
                };
                Ok(fmt.format(&d).to_string())
            }),

            number: Box::new(move |n| {
                let n_str = n.to_string();
                match Decimal::from_str(&n_str) {
                    Ok(d) => f_number.format_to_string(&d),
                    _ => n_str,
                }
            }),

            plural: Box::new(move |count| match plurals.category_for(count.unsigned_abs()) {
                PluralCategory::Zero => "zero",
                PluralCategory::One => "one",
                PluralCategory::Two => "two",
                PluralCategory::Few => "few",
                PluralCategory::Many => "many",
                _ => "other",
            }),

            allow_missing_vars: false,
        };

        Ok(r)
    }

    /// Preserve placeholders verbatim instead of erroring when a var is missing.
    pub const fn allow_missing_vars(mut self) -> Self {
        self.allow_missing_vars = true;
        self
    }
}
