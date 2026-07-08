pub use grand_line::prelude::*;

// Tests for the `find_case` utility function, which powers the plural case scanner.
// `find_case(cases, keyword)` returns the content between `{...}` for the
// first occurrence of `keyword` in the ICU case string, or None.

// ---------------------------------------------------------------------------
// Basic lookups
// ---------------------------------------------------------------------------

#[test]
fn find_case_single_match() {
    pretty_eq!(find_case("one{apple}", "one"), Some("apple"));
}

#[test]
fn find_case_other_case() {
    pretty_eq!(find_case("one{apple} other{apples}", "other"), Some("apples"));
}

#[test]
fn find_case_first_of_two() {
    pretty_eq!(find_case("one{a} other{b}", "one"), Some("a"));
}

#[test]
fn find_case_missing_returns_none() {
    pretty_eq!(find_case("one{apple} other{apples}", "few"), None);
}

#[test]
fn find_case_empty_cases_returns_none() {
    pretty_eq!(find_case("", "one"), None);
}

// ---------------------------------------------------------------------------
// Exact match keywords (=N)
// ---------------------------------------------------------------------------

#[test]
fn find_case_exact_zero() {
    pretty_eq!(find_case("=0{zero} one{one} other{many}", "=0"), Some("zero"));
}

#[test]
fn find_case_exact_one() {
    pretty_eq!(find_case("=1{single} other{plural}", "=1"), Some("single"));
}

#[test]
fn find_case_exact_negative() {
    // Negative exact: unlikely in practice but the scanner allows `-` in keywords.
    pretty_eq!(find_case("=-1{minus one} other{other}", "=-1"), Some("minus one"));
}

// ---------------------------------------------------------------------------
// Empty case body
// ---------------------------------------------------------------------------

#[test]
fn find_case_empty_body() {
    pretty_eq!(find_case("zero{} other{things}", "zero"), Some(""));
}

// ---------------------------------------------------------------------------
// Whitespace handling
// ---------------------------------------------------------------------------

#[test]
fn find_case_leading_whitespace_before_keyword() {
    // Whitespace before a keyword is skipped.
    pretty_eq!(find_case("  one{val}", "one"), Some("val"));
}

#[test]
fn find_case_whitespace_between_cases() {
    // Spaces between `}` and the next keyword are skipped.
    pretty_eq!(find_case("one{A}   other{B}", "other"), Some("B"));
}

#[test]
fn find_case_newline_between_cases() {
    pretty_eq!(find_case("one{A}\nother{B}", "other"), Some("B"));
}

// ---------------------------------------------------------------------------
// Nested braces inside a case body
// ---------------------------------------------------------------------------

#[test]
fn find_case_nested_braces_in_body() {
    // Depth tracking should skip the inner `{}` and not stop at it.
    pretty_eq!(
        find_case("one{has {inner} content} other{plain}", "one"),
        Some("has {inner} content"),
    );
}

#[test]
fn find_case_deeply_nested_braces() {
    pretty_eq!(find_case("other{a {b {c} d} e}", "other"), Some("a {b {c} d} e"));
}

// ---------------------------------------------------------------------------
// The `#` character is preserved as-is (replacement is done by the caller)
// ---------------------------------------------------------------------------

#[test]
fn find_case_hash_in_body_is_preserved() {
    // find_case does not perform # -> count substitution, that is the caller's job.
    pretty_eq!(find_case("other{# items}", "other"), Some("# items"));
}

// ---------------------------------------------------------------------------
// No `{` after keyword -> break (not a valid case, returns None)
// ---------------------------------------------------------------------------

#[test]
fn find_case_keyword_without_brace_returns_none() {
    // "one" is not followed by `{` -> break -> None.
    pretty_eq!(find_case("one other{other}", "one"), None);
}

// ---------------------------------------------------------------------------
// Unclosed brace inside a case body -> break
// ---------------------------------------------------------------------------

#[test]
fn find_case_unclosed_body_brace_returns_none() {
    // The body `{unclosed` never reaches depth=0 -> break -> None.
    pretty_eq!(find_case("other{unclosed", "other"), None);
}
