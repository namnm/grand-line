#![allow(clippy::literal_string_with_formatting_args)]

pub use grand_line::prelude::*;

// ---------------------------------------------------------------------------
// Helper to build expected Placeholder values concisely.
// ---------------------------------------------------------------------------

fn ph(var: &str, fn_name: Option<&str>, args: &[&str]) -> Placeholder {
    Placeholder {
        var: var.to_owned(),
        fn_name: fn_name.map(|v| v.to_owned()),
        args: args.iter().map(|s| (*s).to_owned()).collect(),
    }
}

// ---------------------------------------------------------------------------
// Raw placeholders -- from raw.rs
// ---------------------------------------------------------------------------

#[test]
fn parser_raw_string_var() {
    pretty_eq!(
        intl_parse("{name}"),
        vec![ph("name", None, &[])],
        "string var should parse as a raw placeholder",
    );
}

#[test]
fn parser_raw_int_var() {
    pretty_eq!(
        intl_parse("{n}"),
        vec![ph("n", None, &[])],
        "int var should parse as a raw placeholder",
    );
}

#[test]
fn parser_raw_two_adjacent() {
    pretty_eq!(
        intl_parse("{a}{b}"),
        vec![ph("a", None, &[]), ph("b", None, &[])],
        "two adjacent placeholders should both parse",
    );
}

#[test]
fn parser_raw_same_var_twice() {
    pretty_eq!(
        intl_parse("{a} and {a}"),
        vec![ph("a", None, &[]), ph("a", None, &[])],
        "same var referenced twice should produce two placeholders",
    );
}

#[test]
fn parser_raw_var_at_start() {
    pretty_eq!(
        intl_parse("{who} is here"),
        vec![ph("who", None, &[])],
        "var at the start of the template should parse",
    );
}

#[test]
fn parser_raw_var_at_end() {
    pretty_eq!(
        intl_parse("Hello, {who}"),
        vec![ph("who", None, &[])],
        "var at the end of the template should parse",
    );
}

#[test]
fn parser_raw_missing_produces_same_shape() {
    // Whether or not the var is supplied to intl, the shape is the same.
    pretty_eq!(
        intl_parse("{name}"),
        vec![ph("name", None, &[])],
        "missing var should still parse to the same placeholder shape",
    );
}

#[test]
fn parser_raw_numeric_var_name() {
    // All-digit names are alphanumeric, so they parse fine.
    pretty_eq!(
        intl_parse("{0}"),
        vec![ph("0", None, &[])],
        "single digit var name should parse as a valid placeholder",
    );
    pretty_eq!(
        intl_parse("{42}"),
        vec![ph("42", None, &[])],
        "multi digit var name should parse as a valid placeholder",
    );
}

// ---------------------------------------------------------------------------
// Date placeholders -- from date.rs
// ---------------------------------------------------------------------------

#[test]
fn parser_date_no_style() {
    pretty_eq!(
        intl_parse("{d, date}"),
        vec![ph("d", Some("date"), &[])],
        "date without a style should parse with no args",
    );
}

#[test]
fn parser_date_short() {
    pretty_eq!(
        intl_parse("{d, date, short}"),
        vec![ph("d", Some("date"), &["short"])],
        "date with short style should parse the style as an arg",
    );
}

#[test]
fn parser_date_long() {
    pretty_eq!(
        intl_parse("{d, date, long}"),
        vec![ph("d", Some("date"), &["long"])],
        "date with long style should parse the style as an arg",
    );
}

#[test]
fn parser_date_full() {
    pretty_eq!(
        intl_parse("{d, date, full}"),
        vec![ph("d", Some("date"), &["full"])],
        "date with full style should parse the style as an arg",
    );
}

#[test]
fn parser_date_medium() {
    // "medium" is the fallback in intl but still parses as an arg.
    pretty_eq!(
        intl_parse("{d, date, medium}"),
        vec![ph("d", Some("date"), &["medium"])],
        "date with medium style should still parse the style as an arg",
    );
}

#[test]
fn parser_date_unknown_style() {
    pretty_eq!(
        intl_parse("{d, date, whatever}"),
        vec![ph("d", Some("date"), &["whatever"])],
        "date with an unrecognised style should still parse the style as an arg",
    );
}

#[test]
fn parser_time_no_style() {
    pretty_eq!(
        intl_parse("{t, time}"),
        vec![ph("t", Some("time"), &[])],
        "time placeholder should parse with fn_name time",
    );
}

#[test]
fn parser_date_in_sentence() {
    pretty_eq!(
        intl_parse("Member since {joined, date, short}"),
        vec![ph("joined", Some("date"), &["short"])],
        "date placeholder embedded in a sentence should parse",
    );
}

// ---------------------------------------------------------------------------
// Number placeholders -- from number.rs
// ---------------------------------------------------------------------------

#[test]
fn parser_number() {
    pretty_eq!(
        intl_parse("{amount, number}"),
        vec![ph("amount", Some("number"), &[])],
        "number placeholder should parse with fn_name number",
    );
}

#[test]
fn parser_number_in_sentence() {
    pretty_eq!(
        intl_parse("Total: {amount, number}"),
        vec![ph("amount", Some("number"), &[])],
        "number placeholder embedded in a sentence should parse",
    );
}

// ---------------------------------------------------------------------------
// Plural placeholders -- from plural.rs
// ---------------------------------------------------------------------------

#[test]
fn parser_plural_one_other() {
    pretty_eq!(
        intl_parse("{c, plural, one{# item} other{# items}}"),
        vec![ph("c", Some("plural"), &["one{# item}", "other{# items}"])],
        "plural with one and other cases should parse both cases as args",
    );
}

#[test]
fn parser_plural_exact_zero_override() {
    pretty_eq!(
        intl_parse("{c, plural, =0{No messages} one{# message} other{# messages}}"),
        vec![ph(
            "c",
            Some("plural"),
            &["=0{No messages}", "one{# message}", "other{# messages}"],
        )],
        "plural with an exact zero case should preserve case order",
    );
}

#[test]
fn parser_plural_exact_one_override() {
    pretty_eq!(
        intl_parse("{c, plural, =1{Exactly one!} one{# message} other{# messages}}"),
        vec![ph(
            "c",
            Some("plural"),
            &["=1{Exactly one!}", "one{# message}", "other{# messages}"],
        )],
        "plural with an exact one case should preserve case order",
    );
}

#[test]
fn parser_plural_exact_two_three() {
    pretty_eq!(
        intl_parse("{c, plural, =2{pair} =3{triple} other{# things}}"),
        vec![ph("c", Some("plural"), &["=2{pair}", "=3{triple}", "other{# things}"])],
        "plural with exact two and three cases should preserve case order",
    );
}

#[test]
fn parser_plural_only_other() {
    pretty_eq!(
        intl_parse("{c, plural, other{# things}}"),
        vec![ph("c", Some("plural"), &["other{# things}"])],
        "plural with only an other case should parse",
    );
}

#[test]
fn parser_plural_hash_in_body() {
    // # is preserved literally by the parser, substitution is done at format time.
    pretty_eq!(
        intl_parse("{n, plural, one{# apple} other{# apples}}"),
        vec![ph("n", Some("plural"), &["one{# apple}", "other{# apples}"])],
        "hash in a plural case body should be preserved literally",
    );
}

#[test]
fn parser_plural_negative_count() {
    pretty_eq!(
        intl_parse("{n, plural, one{# point} other{# points}}"),
        vec![ph("n", Some("plural"), &["one{# point}", "other{# points}"])],
        "plural placeholder should parse the same regardless of the count sign",
    );
}

#[test]
fn parser_plural_whitespace_between_cases() {
    // Extra whitespace between cases is ignored by the scanner.
    pretty_eq!(
        intl_parse("{c, plural,   one{# item}   other{# items}}"),
        vec![ph("c", Some("plural"), &["one{# item}", "other{# items}"])],
        "extra whitespace between plural cases should be ignored",
    );
}

// ---------------------------------------------------------------------------
// Edge cases -- from edge.rs
// ---------------------------------------------------------------------------

#[test]
fn parser_edge_empty_template() {
    pretty_eq!(intl_parse(""), vec![], "empty template should produce no placeholders");
}

#[test]
fn parser_edge_no_placeholders() {
    pretty_eq!(
        intl_parse("just a plain string"),
        vec![],
        "template without placeholders should produce no placeholders",
    );
}

#[test]
fn parser_edge_whitespace_only() {
    pretty_eq!(
        intl_parse("   "),
        vec![],
        "whitespace only template should produce no placeholders",
    );
}

#[test]
fn parser_edge_unclosed_brace_stops_scan() {
    // Unclosed `{` causes the scanner to stop, nothing is collected.
    pretty_eq!(
        intl_parse("start {unclosed"),
        vec![],
        "unclosed brace should stop the scan and produce no placeholders",
    );
}

#[test]
fn parser_edge_text_before_unclosed_brace() {
    // Text before the placeholder with a valid name, then unclosed.
    pretty_eq!(
        intl_parse("prefix {bad"),
        vec![],
        "text before an unclosed brace should still produce no placeholders",
    );
}

#[test]
fn parser_edge_space_in_var_name_skipped() {
    // "not valid" contains a space -> not alphanumeric -> skipped.
    pretty_eq!(
        intl_parse("{not valid}"),
        vec![],
        "var name with a space should be skipped",
    );
}

#[test]
fn parser_edge_empty_var_name_skipped() {
    // `{}` has an empty var name -> skipped.
    pretty_eq!(intl_parse("{}"), vec![], "empty var name should be skipped");
}

#[test]
fn parser_edge_spaces_around_var_name_trimmed() {
    // cut() trims both sides, so "{ name }" parses as var="name".
    pretty_eq!(
        intl_parse("{ name }"),
        vec![ph("name", None, &[])],
        "spaces around var name should be trimmed",
    );
}

#[test]
fn parser_edge_spaces_around_type_trimmed() {
    pretty_eq!(
        intl_parse("{ n , number }"),
        vec![ph("n", Some("number"), &[])],
        "spaces around type should be trimmed",
    );
}

#[test]
fn parser_edge_unknown_type_preserved() {
    // "currency" is not a recognised formatter but we still record it.
    pretty_eq!(
        intl_parse("{amount, currency}"),
        vec![ph("amount", Some("currency"), &[])],
        "unknown formatter type should still be recorded as fn_name",
    );
}

#[test]
fn parser_edge_unknown_type_with_tail() {
    pretty_eq!(
        intl_parse("{v, money, USD}"),
        vec![ph("v", Some("money"), &["USD"])],
        "unknown formatter type with a tail should record the tail as an arg",
    );
}

#[test]
fn parser_edge_partial_var_resolution() {
    // intl_parse is purely structural -- it does not care which vars exist.
    pretty_eq!(
        intl_parse("{a} {b} {c}"),
        vec![ph("a", None, &[]), ph("b", None, &[]), ph("c", None, &[])],
        "parser should be structural and list every placeholder regardless of missing vars",
    );
}

#[test]
fn parser_edge_plural_with_inner_braces() {
    // Depth tracking should handle `{A}` inside the plural body without
    // confusing the outer closing `}`.
    pretty_eq!(
        intl_parse("{c, plural, one{section {A}} other{# sections}}"),
        vec![ph("c", Some("plural"), &["one{section {A}}", "other{# sections}"])],
        "nested braces in a plural case body should not confuse depth tracking",
    );
}

// ---------------------------------------------------------------------------
// Mixed templates -- from mixed.rs
// ---------------------------------------------------------------------------

#[test]
fn parser_mixed_member_since() {
    pretty_eq!(
        intl_parse("Member since {joined, date, short}"),
        vec![ph("joined", Some("date"), &["short"])],
        "member since template should parse to a single date placeholder",
    );
}

#[test]
fn parser_mixed_invoice() {
    pretty_eq!(
        intl_parse("Invoice #{id} - Total: {amount, number} - Due: {due, date, long}"),
        vec![
            ph("id", None, &[]),
            ph("amount", Some("number"), &[]),
            ph("due", Some("date"), &["long"]),
        ],
        "invoice template should parse all three placeholders in order",
    );
}

#[test]
fn parser_mixed_greeting_with_count() {
    pretty_eq!(
        intl_parse("Hello {name}! You have {n, plural, one{# message} other{# messages}}."),
        vec![
            ph("name", None, &[]),
            ph("n", Some("plural"), &["one{# message}", "other{# messages}"]),
        ],
        "greeting with unread count should parse both placeholders",
    );
}

#[test]
fn parser_mixed_plural_and_date() {
    pretty_eq!(
        intl_parse("{n, plural, one{# order} other{# orders}} since {d, date, long}"),
        vec![
            ph("n", Some("plural"), &["one{# order}", "other{# orders}"]),
            ph("d", Some("date"), &["long"]),
        ],
        "plural and date combined should parse both placeholders",
    );
}

#[test]
fn parser_mixed_number_and_plural() {
    pretty_eq!(
        intl_parse("Price: {price, number} ({qty, plural, one{# unit} other{# units}})"),
        vec![
            ph("price", Some("number"), &[]),
            ph("qty", Some("plural"), &["one{# unit}", "other{# units}"]),
        ],
        "number and plural combined should parse both placeholders",
    );
}

#[test]
fn parser_mixed_three_types() {
    pretty_eq!(
        intl_parse("{name} paid {amount, number} on {date, date, short}"),
        vec![
            ph("name", None, &[]),
            ph("amount", Some("number"), &[]),
            ph("date", Some("date"), &["short"]),
        ],
        "three different placeholder types should all parse",
    );
}

#[test]
fn parser_mixed_same_var_twice() {
    pretty_eq!(
        intl_parse("User {user} logged in as {user}"),
        vec![ph("user", None, &[]), ph("user", None, &[])],
        "same var referenced twice should produce two placeholders",
    );
}

#[test]
fn parser_mixed_some_vars_missing() {
    // Parser is structural, missing vs present vars produce the same output.
    pretty_eq!(
        intl_parse("{name} owes {amount, number} by {due, date}"),
        vec![
            ph("name", None, &[]),
            ph("amount", Some("number"), &[]),
            ph("due", Some("date"), &[]),
        ],
        "missing vars should not affect parsing since it is purely structural",
    );
}

#[test]
fn parser_mixed_notification() {
    pretty_eq!(
        intl_parse("{actor} commented on your post from {date, date, long} ({n, plural, one{# like} other{# likes}})",),
        vec![
            ph("actor", None, &[]),
            ph("date", Some("date"), &["long"]),
            ph("n", Some("plural"), &["one{# like}", "other{# likes}"]),
        ],
        "notification template should parse all three placeholders",
    );
}

#[test]
fn parser_mixed_time_and_number() {
    pretty_eq!(
        intl_parse("Event at {t, time} - {seats, number} seats remaining"),
        vec![ph("t", Some("time"), &[]), ph("seats", Some("number"), &[])],
        "time and number combined should parse both placeholders",
    );
}
