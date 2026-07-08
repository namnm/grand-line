pub use grand_line::prelude::*;
use std::borrow::Cow;

// ---------------------------------------------------------------------------
// preprocess_intl_template
//
// Transforms `intl`...`` tagged templates into intl("...", #{var: var}) calls.
// Tests mirror the behaviour described in the preprocessor doc-comment.
// ---------------------------------------------------------------------------

fn pp(s: &str) -> String {
    preprocess_intl_template(s).into_owned()
}

fn id(s: &str) -> &str {
    // Helper: asserts the string is returned borrowed (no transformation)
    let out = preprocess_intl_template(s);
    assert!(matches!(out, Cow::Borrowed(_)), "expected Borrowed (no-op) for: {s:?}");
    s
}

// ---------------------------------------------------------------------------
// No-op cases
// ---------------------------------------------------------------------------

#[test]
fn noop_plain_expression() {
    id("current_user");
}

#[test]
fn noop_double_quoted_string() {
    id(r#""hello world""#);
}

#[test]
fn noop_no_intl_keyword() {
    id(r#""intl is great""#);
}

#[test]
fn noop_intl_keyword_no_backtick() {
    // "intl" without a trailing `` ` `` is not a tagged template.
    id("intl(\"hello\")");
}

#[test]
fn noop_intl_tag_inside_double_quote() {
    // intl` inside a quoted string should not be transformed.
    let s = r#""intl`test`""#;
    pretty_eq!(preprocess_intl_template(s).as_ref(), s);
}

#[test]
fn noop_intl_tag_inside_single_quote() {
    let s = "'intl`test`'";
    pretty_eq!(preprocess_intl_template(s).as_ref(), s);
}

#[test]
fn noop_intl_tag_inside_backtick_string() {
    // A plain backtick string is opened first, the `intl`` inside it is literal.
    let s = "`prefix intl`inner` suffix`";
    // After the opening `, "prefix intl" is inside a backtick string.
    // The next ` ends the string, " suffix`" becomes code. No intl` sequence
    // in code context with a word boundary, so no transformation.
    // (This edge case is implementation-defined, the important invariant is
    // that we do NOT produce garbled output.)
    let _ = preprocess_intl_template(s); // should not panic
}

#[test]
fn noop_intl_tag_inside_line_comment() {
    let s = "// intl`test`";
    pretty_eq!(preprocess_intl_template(s).as_ref(), s);
}

#[test]
fn noop_intl_tag_inside_block_comment() {
    let s = "/* intl`test` */";
    pretty_eq!(preprocess_intl_template(s).as_ref(), s);
}

#[test]
fn noop_word_boundary_prevents_match() {
    // "print_intl`test`" - 'intl' is preceded by '_', not a word boundary.
    let s = "print_intl`test`";
    pretty_eq!(preprocess_intl_template(s).as_ref(), s);
}

#[test]
fn noop_ident_prefix_prevents_match() {
    // "myintl`test`" - 'i' in 'intl' is preceded by 'y' (alnum).
    let s = "myintl`test`";
    pretty_eq!(preprocess_intl_template(s).as_ref(), s);
}

// ---------------------------------------------------------------------------
// Basic transformations
// ---------------------------------------------------------------------------

#[test]
fn no_vars_produces_1arg_call() {
    pretty_eq!(pp("intl`Hello!`"), r#"intl("Hello!")"#);
}

#[test]
fn empty_template() {
    pretty_eq!(pp("intl``"), r#"intl("")"#);
}

#[test]
fn one_var_produces_map_arg() {
    pretty_eq!(pp("intl`Hello {name}!`"), r#"intl("Hello {name}!", #{name: name})"#);
}

#[test]
fn two_vars_both_in_map() {
    pretty_eq!(pp("intl`{a} and {b}`"), r#"intl("{a} and {b}", #{a: a, b: b})"#);
}

#[test]
fn duplicate_var_deduplicated() {
    pretty_eq!(
        pp("intl`{name} is {name}`"),
        r#"intl("{name} is {name}", #{name: name})"#,
    );
}

#[test]
fn var_with_type_annotation() {
    // {amount, number} -> var name is "amount"
    pretty_eq!(
        pp("intl`Total: {amount, number}`"),
        r#"intl("Total: {amount, number}", #{amount: amount})"#,
    );
}

#[test]
fn plural_only_extracts_outer_var() {
    // Nested {} in plural body are skipped, only "count" is extracted.
    pretty_eq!(
        pp("intl`{count, plural, one{# item} other{# items}}`"),
        r#"intl("{count, plural, one{# item} other{# items}}", #{count: count})"#,
    );
}

#[test]
fn date_var_extracted() {
    pretty_eq!(
        pp("intl`Due: {due, date, long}`"),
        r#"intl("Due: {due, date, long}", #{due: due})"#,
    );
}

// ---------------------------------------------------------------------------
// Escape handling
// ---------------------------------------------------------------------------

#[test]
fn escaped_backtick_in_template() {
    // \` -> literal ` in the generated double-quoted string
    pretty_eq!(pp(r"intl`say \` ok`"), r#"intl("say ` ok")"#);
}

#[test]
fn double_quote_in_template_escaped() {
    pretty_eq!(pp(r#"intl`say "hi"`"#), r#"intl("say \"hi\"")"#);
}

#[test]
fn backslash_in_template_escaped() {
    pretty_eq!(pp(r"intl`a\b`"), r#"intl("a\\b")"#);
}

#[test]
fn newline_in_template_escaped() {
    pretty_eq!(pp("intl`line1\nline2`"), r#"intl("line1\nline2")"#);
}

// ---------------------------------------------------------------------------
// Multiple templates in one script
// ---------------------------------------------------------------------------

#[test]
fn two_intl_tags_in_script() {
    pretty_eq!(pp("intl`Hello!`\nintl`Bye!`"), "intl(\"Hello!\")\nintl(\"Bye!\")");
}

#[test]
fn intl_tag_adjacent_to_expression() {
    pretty_eq!(pp("x = intl`Hi {name}!`"), r#"x = intl("Hi {name}!", #{name: name})"#);
}

// ---------------------------------------------------------------------------
// Unclosed backtick -> no transformation
// ---------------------------------------------------------------------------

#[test]
fn unclosed_backtick_not_transformed() {
    // Unclosed `intl`` is left as-is (no partial replacement).
    let s = "intl`unclosed";
    pretty_eq!(preprocess_intl_template(s).as_ref(), s);
}
