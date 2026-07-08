pub use grand_line::prelude::*;

#[path = "./prelude.rs"]
mod prelude;
use prelude::*;

// ---------------------------------------------------------------------------
// Basic plural categories (English: one / other)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_one_en() -> Res<()> {
    let tpl = "You have {c, plural, one{# message} other{# messages}}.";
    intl(tpl, &[("c", IntlValue::Int(1))], "You have 1 message.")
}

#[tokio::test]
async fn plural_other_en() -> Res<()> {
    let tpl = "You have {c, plural, one{# message} other{# messages}}.";
    intl(tpl, &[("c", IntlValue::Int(5))], "You have 5 messages.")
}

#[tokio::test]
async fn plural_two_uses_other() -> Res<()> {
    // English has no "two" category, 2 -> "other".
    let tpl = "{c, plural, one{# item} other{# items}}";
    intl(tpl, &[("c", IntlValue::Int(2))], "2 items")
}

#[tokio::test]
async fn plural_zero_uses_other() -> Res<()> {
    // English: 0 -> "other" (no "zero" category in EN cardinal rules).
    let tpl = "{c, plural, one{# item} other{# items}}";
    intl(tpl, &[("c", IntlValue::Int(0))], "0 items")
}

#[tokio::test]
async fn plural_large_count_uses_other() -> Res<()> {
    let tpl = "{c, plural, one{# item} other{# items}}";
    intl(tpl, &[("c", IntlValue::Int(1_000))], "1000 items")
}

// ---------------------------------------------------------------------------
// Exact matches (=N) take priority over category
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_exact_zero_en() -> Res<()> {
    let tpl = "{c, plural, =0{No messages} one{# message} other{# messages}}.";
    intl(tpl, &[("c", IntlValue::Int(0))], "No messages.")
}

#[tokio::test]
async fn plural_exact_one_overrides_category() -> Res<()> {
    // =1{} should be selected before the "one" category.
    let tpl = "{c, plural, =1{exactly one} one{one cat} other{# others}}";
    intl(tpl, &[("c", IntlValue::Int(1))], "exactly one")
}

#[tokio::test]
async fn plural_exact_two() -> Res<()> {
    let tpl = "{c, plural, =2{a pair} one{# item} other{# items}}";
    intl(tpl, &[("c", IntlValue::Int(2))], "a pair")
}

#[tokio::test]
async fn plural_exact_three() -> Res<()> {
    let tpl = "{c, plural, =3{triple} other{# items}}";
    intl(tpl, &[("c", IntlValue::Int(3))], "triple")
}

// ---------------------------------------------------------------------------
// Hash (#) substitution in case body
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_hash_replaced_with_count() -> Res<()> {
    let tpl = "{c, plural, one{# apple} other{# apples}}";
    intl(tpl, &[("c", IntlValue::Int(42))], "42 apples")
}

#[tokio::test]
async fn plural_hash_in_sentence_case() -> Res<()> {
    let tpl = "{c, plural, one{# result found} other{# results found}}";
    intl(tpl, &[("c", IntlValue::Int(1))], "1 result found")
}

// ---------------------------------------------------------------------------
// Negative counts: unsigned_abs used for category lookup
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_negative_uses_other() -> Res<()> {
    // -5 -> abs=5 -> "other" in English
    let tpl = "{c, plural, one{# item} other{# items}}";
    intl(tpl, &[("c", IntlValue::Int(-5))], "-5 items")
}

// ---------------------------------------------------------------------------
// Fallback when no case matches and no "other"
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_no_matching_case_falls_to_count() -> Res<()> {
    // Only "one" case, count=5 -> "other" -> not found -> count.to_string().
    let tpl = "{c, plural, one{single}}";
    intl(tpl, &[("c", IntlValue::Int(5))], "5")
}

// ---------------------------------------------------------------------------
// Only "other" case
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_only_other_case() -> Res<()> {
    let tpl = "{c, plural, other{# thing(s)}}";
    intl(tpl, &[("c", IntlValue::Int(1))], "1 thing(s)")
}

// ---------------------------------------------------------------------------
// Whitespace tolerance between cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_whitespace_between_cases() -> Res<()> {
    // Spaces between cases should be handled correctly.
    let tpl = "{c, plural,   one{# item}   other{# items}}";
    intl(tpl, &[("c", IntlValue::Int(3))], "3 items")
}

// ---------------------------------------------------------------------------
// Missing variable -> placeholder preserved
// ---------------------------------------------------------------------------

#[tokio::test]
async fn plural_missing_var_preserved() -> Res<()> {
    let tpl = "{c, plural, one{# message} other{# messages}}";
    intl_no_vars(tpl, tpl)
}
