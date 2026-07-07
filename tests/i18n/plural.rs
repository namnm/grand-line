#![allow(clippy::literal_string_with_formatting_args)]

pub use grand_line::prelude::*;

#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// Basic plural categories (English: one / other)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_one_en() -> Res<()> {
    let tpl = "You have {c, plural, one{# message} other{# messages}}.";
    let v = hashmap! {
        "c" => IntlValue::Int(1),
    };
    intl_assert(tpl, &v, "You have 1 message.")
}

#[tokio::test]
async fn plural_other_en() -> Res<()> {
    let tpl = "You have {c, plural, one{# message} other{# messages}}.";
    let v = hashmap! {
        "c" => IntlValue::Int(5),
    };
    intl_assert(tpl, &v, "You have 5 messages.")
}

#[tokio::test]
async fn plural_two_uses_other() -> Res<()> {
    // English has no "two" category, 2 -> "other".
    let tpl = "{c, plural, one{# item} other{# items}}";
    let v = hashmap! {
        "c" => IntlValue::Int(2),
    };
    intl_assert(tpl, &v, "2 items")
}

#[tokio::test]
async fn plural_zero_uses_other() -> Res<()> {
    // English: 0 -> "other" (no "zero" category in EN cardinal rules).
    let tpl = "{c, plural, one{# item} other{# items}}";
    let v = hashmap! {
        "c" => IntlValue::Int(0),
    };
    intl_assert(tpl, &v, "0 items")
}

#[tokio::test]
async fn plural_large_count_uses_other() -> Res<()> {
    let tpl = "{c, plural, one{# item} other{# items}}";
    let v = hashmap! {
        "c" => IntlValue::Int(1_000),
    };
    intl_assert(tpl, &v, "1000 items")
}

// ---------------------------------------------------------------------------
// Exact matches (=N) take priority over category
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_exact_zero_en() -> Res<()> {
    let tpl = "{c, plural, =0{No messages} one{# message} other{# messages}}.";
    let v = hashmap! {
        "c" => IntlValue::Int(0),
    };
    intl_assert(tpl, &v, "No messages.")
}

#[tokio::test]
async fn plural_exact_one_overrides_category() -> Res<()> {
    let tpl = "{c, plural, =1{exactly one} one{one cat} other{# others}}";
    let v = hashmap! {
        "c" => IntlValue::Int(1),
    };
    intl_assert(tpl, &v, "exactly one")
}

#[tokio::test]
async fn plural_exact_two() -> Res<()> {
    let tpl = "{c, plural, =2{a pair} one{# item} other{# items}}";
    let v = hashmap! {
        "c" => IntlValue::Int(2),
    };
    intl_assert(tpl, &v, "a pair")
}

#[tokio::test]
async fn plural_exact_three() -> Res<()> {
    let tpl = "{c, plural, =3{triple} other{# items}}";
    let v = hashmap! {
        "c" => IntlValue::Int(3),
    };
    intl_assert(tpl, &v, "triple")
}

// ---------------------------------------------------------------------------
// Hash (#) substitution in case body
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_hash_replaced_with_count() -> Res<()> {
    let tpl = "{c, plural, one{# apple} other{# apples}}";
    let v = hashmap! {
        "c" => IntlValue::Int(42),
    };
    intl_assert(tpl, &v, "42 apples")
}

#[tokio::test]
async fn plural_hash_in_sentence_case() -> Res<()> {
    let tpl = "{c, plural, one{# result found} other{# results found}}";
    let v = hashmap! {
        "c" => IntlValue::Int(1),
    };
    intl_assert(tpl, &v, "1 result found")
}

// ---------------------------------------------------------------------------
// Negative counts: unsigned_abs used for category lookup
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_negative_uses_other() -> Res<()> {
    let tpl = "{c, plural, one{# item} other{# items}}";
    let v = hashmap! {
        "c" => IntlValue::Int(-5),
    };
    intl_assert(tpl, &v, "-5 items")
}

// ---------------------------------------------------------------------------
// Fallback when no case matches and no "other"
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_no_matching_case_falls_to_count() -> Res<()> {
    let tpl = "{c, plural, one{single}}";
    let v = hashmap! {
        "c" => IntlValue::Int(5),
    };
    intl_assert(tpl, &v, "5")
}

// ---------------------------------------------------------------------------
// Only "other" case
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_only_other_case() -> Res<()> {
    let tpl = "{c, plural, other{# thing(s)}}";
    let v = hashmap! {
        "c" => IntlValue::Int(1),
    };
    intl_assert(tpl, &v, "1 thing(s)")
}

// ---------------------------------------------------------------------------
// Whitespace tolerance between cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_whitespace_between_cases() -> Res<()> {
    let tpl = "{c, plural,   one{# item}   other{# items}}";
    let v = hashmap! {
        "c" => IntlValue::Int(3),
    };
    intl_assert(tpl, &v, "3 items")
}

// ---------------------------------------------------------------------------
// Missing variable -> placeholder preserved
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_missing_var_preserved() -> Res<()> {
    let tpl = "{c, plural, one{# message} other{# messages}}";
    let v = hashmap! {};
    intl_assert_lenient(tpl, &v, tpl)
}

// ---------------------------------------------------------------------------
// Wrong value type (a string where a number is required) returns Err
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_str_value_returns_err() -> Res<()> {
    let tpl = "{c, plural, one{# message} other{# messages}}";
    let v = hashmap! {
        "c" => IntlValue::Str("not a count".into()),
    };
    let ctx = ctx("en")?;
    let r = intl(tpl, &v, &ctx);
    pretty_eq!(
        r.is_err(),
        true,
        "a string value for a plural placeholder should return an error",
    );
    Ok(())
}
