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
    pretty_eq!(parse_template("{name}"), vec![ph("name", None, &[])]);
}

#[test]
fn parser_raw_int_var() {
    pretty_eq!(parse_template("{n}"), vec![ph("n", None, &[])]);
}

#[test]
fn parser_raw_two_adjacent() {
    pretty_eq!(parse_template("{a}{b}"), vec![ph("a", None, &[]), ph("b", None, &[])]);
}

#[test]
fn parser_raw_same_var_twice() {
    pretty_eq!(
        parse_template("{a} and {a}"),
        vec![ph("a", None, &[]), ph("a", None, &[])],
    );
}

#[test]
fn parser_raw_var_at_start() {
    pretty_eq!(parse_template("{who} is here"), vec![ph("who", None, &[])]);
}

#[test]
fn parser_raw_var_at_end() {
    pretty_eq!(parse_template("Hello, {who}"), vec![ph("who", None, &[])]);
}

#[test]
fn parser_raw_missing_produces_same_shape() {
    // Whether or not the var is supplied to format_template, the shape is the same.
    pretty_eq!(parse_template("{name}"), vec![ph("name", None, &[])]);
}

#[test]
fn parser_raw_numeric_var_name() {
    // All-digit names are alphanumeric, so they parse fine.
    pretty_eq!(parse_template("{0}"), vec![ph("0", None, &[])]);
    pretty_eq!(parse_template("{42}"), vec![ph("42", None, &[])]);
}

// ---------------------------------------------------------------------------
// Date placeholders -- from date.rs
// ---------------------------------------------------------------------------

#[test]
fn parser_date_no_style() {
    pretty_eq!(parse_template("{d, date}"), vec![ph("d", Some("date"), &[])]);
}

#[test]
fn parser_date_short() {
    pretty_eq!(
        parse_template("{d, date, short}"),
        vec![ph("d", Some("date"), &["short"])],
    );
}

#[test]
fn parser_date_long() {
    pretty_eq!(
        parse_template("{d, date, long}"),
        vec![ph("d", Some("date"), &["long"])],
    );
}

#[test]
fn parser_date_full() {
    pretty_eq!(
        parse_template("{d, date, full}"),
        vec![ph("d", Some("date"), &["full"])],
    );
}

#[test]
fn parser_date_medium() {
    // "medium" is the fallback in format_template but still parses as an arg.
    pretty_eq!(
        parse_template("{d, date, medium}"),
        vec![ph("d", Some("date"), &["medium"])],
    );
}

#[test]
fn parser_date_unknown_style() {
    pretty_eq!(
        parse_template("{d, date, whatever}"),
        vec![ph("d", Some("date"), &["whatever"])],
    );
}

#[test]
fn parser_time_no_style() {
    pretty_eq!(parse_template("{t, time}"), vec![ph("t", Some("time"), &[])]);
}

#[test]
fn parser_date_in_sentence() {
    pretty_eq!(
        parse_template("Member since {joined, date, short}"),
        vec![ph("joined", Some("date"), &["short"])],
    );
}

// ---------------------------------------------------------------------------
// Number placeholders -- from number.rs
// ---------------------------------------------------------------------------

#[test]
fn parser_number() {
    pretty_eq!(
        parse_template("{amount, number}"),
        vec![ph("amount", Some("number"), &[])],
    );
}

#[test]
fn parser_number_in_sentence() {
    pretty_eq!(
        parse_template("Total: {amount, number}"),
        vec![ph("amount", Some("number"), &[])],
    );
}

// ---------------------------------------------------------------------------
// Plural placeholders -- from plural.rs
// ---------------------------------------------------------------------------

#[test]
fn parser_plural_one_other() {
    pretty_eq!(
        parse_template("{c, plural, one{# item} other{# items}}"),
        vec![ph("c", Some("plural"), &["one{# item}", "other{# items}"])],
    );
}

#[test]
fn parser_plural_exact_zero_override() {
    pretty_eq!(
        parse_template("{c, plural, =0{No messages} one{# message} other{# messages}}"),
        vec![ph(
            "c",
            Some("plural"),
            &["=0{No messages}", "one{# message}", "other{# messages}"],
        )],
    );
}

#[test]
fn parser_plural_exact_one_override() {
    pretty_eq!(
        parse_template("{c, plural, =1{Exactly one!} one{# message} other{# messages}}"),
        vec![ph(
            "c",
            Some("plural"),
            &["=1{Exactly one!}", "one{# message}", "other{# messages}"],
        )],
    );
}

#[test]
fn parser_plural_exact_two_three() {
    pretty_eq!(
        parse_template("{c, plural, =2{pair} =3{triple} other{# things}}"),
        vec![ph("c", Some("plural"), &["=2{pair}", "=3{triple}", "other{# things}"])],
    );
}

#[test]
fn parser_plural_only_other() {
    pretty_eq!(
        parse_template("{c, plural, other{# things}}"),
        vec![ph("c", Some("plural"), &["other{# things}"])],
    );
}

#[test]
fn parser_plural_hash_in_body() {
    // # is preserved literally by the parser, substitution is done at format time.
    pretty_eq!(
        parse_template("{n, plural, one{# apple} other{# apples}}"),
        vec![ph("n", Some("plural"), &["one{# apple}", "other{# apples}"])],
    );
}

#[test]
fn parser_plural_negative_count() {
    pretty_eq!(
        parse_template("{n, plural, one{# point} other{# points}}"),
        vec![ph("n", Some("plural"), &["one{# point}", "other{# points}"])],
    );
}

#[test]
fn parser_plural_whitespace_between_cases() {
    // Extra whitespace between cases is ignored by the scanner.
    pretty_eq!(
        parse_template("{c, plural,   one{# item}   other{# items}}"),
        vec![ph("c", Some("plural"), &["one{# item}", "other{# items}"])],
    );
}

// ---------------------------------------------------------------------------
// Edge cases -- from edge.rs
// ---------------------------------------------------------------------------

#[test]
fn parser_edge_empty_template() {
    pretty_eq!(parse_template(""), vec![]);
}

#[test]
fn parser_edge_no_placeholders() {
    pretty_eq!(parse_template("just a plain string"), vec![]);
}

#[test]
fn parser_edge_whitespace_only() {
    pretty_eq!(parse_template("   "), vec![]);
}

#[test]
fn parser_edge_unclosed_brace_stops_scan() {
    // Unclosed `{` causes the scanner to stop, nothing is collected.
    pretty_eq!(parse_template("start {unclosed"), vec![]);
}

#[test]
fn parser_edge_text_before_unclosed_brace() {
    // Text before the placeholder with a valid name, then unclosed.
    pretty_eq!(parse_template("prefix {bad"), vec![]);
}

#[test]
fn parser_edge_space_in_var_name_skipped() {
    // "not valid" contains a space -> not alphanumeric -> skipped.
    pretty_eq!(parse_template("{not valid}"), vec![]);
}

#[test]
fn parser_edge_empty_var_name_skipped() {
    // `{}` has an empty var name -> skipped.
    pretty_eq!(parse_template("{}"), vec![]);
}

#[test]
fn parser_edge_spaces_around_var_name_trimmed() {
    // cut() trims both sides, so "{ name }" parses as var="name".
    pretty_eq!(parse_template("{ name }"), vec![ph("name", None, &[])]);
}

#[test]
fn parser_edge_spaces_around_type_trimmed() {
    pretty_eq!(parse_template("{ n , number }"), vec![ph("n", Some("number"), &[])]);
}

#[test]
fn parser_edge_unknown_type_preserved() {
    // "currency" is not a recognised formatter but we still record it.
    pretty_eq!(
        parse_template("{amount, currency}"),
        vec![ph("amount", Some("currency"), &[])],
    );
}

#[test]
fn parser_edge_unknown_type_with_tail() {
    pretty_eq!(
        parse_template("{v, money, USD}"),
        vec![ph("v", Some("money"), &["USD"])],
    );
}

#[test]
fn parser_edge_partial_var_resolution() {
    // parse_template is purely structural -- it does not care which vars exist.
    pretty_eq!(
        parse_template("{a} {b} {c}"),
        vec![ph("a", None, &[]), ph("b", None, &[]), ph("c", None, &[])],
    );
}

#[test]
fn parser_edge_plural_with_inner_braces() {
    // Depth tracking should handle `{A}` inside the plural body without
    // confusing the outer closing `}`.
    pretty_eq!(
        parse_template("{c, plural, one{section {A}} other{# sections}}"),
        vec![ph("c", Some("plural"), &["one{section {A}}", "other{# sections}"])],
    );
}

// ---------------------------------------------------------------------------
// Mixed templates -- from mixed.rs
// ---------------------------------------------------------------------------

#[test]
fn parser_mixed_member_since() {
    pretty_eq!(
        parse_template("Member since {joined, date, short}"),
        vec![ph("joined", Some("date"), &["short"])],
    );
}

#[test]
fn parser_mixed_invoice() {
    pretty_eq!(
        parse_template("Invoice #{id} - Total: {amount, number} - Due: {due, date, long}"),
        vec![
            ph("id", None, &[]),
            ph("amount", Some("number"), &[]),
            ph("due", Some("date"), &["long"]),
        ],
    );
}

#[test]
fn parser_mixed_greeting_with_count() {
    pretty_eq!(
        parse_template("Hello {name}! You have {n, plural, one{# message} other{# messages}}."),
        vec![
            ph("name", None, &[]),
            ph("n", Some("plural"), &["one{# message}", "other{# messages}"]),
        ],
    );
}

#[test]
fn parser_mixed_plural_and_date() {
    pretty_eq!(
        parse_template("{n, plural, one{# order} other{# orders}} since {d, date, long}"),
        vec![
            ph("n", Some("plural"), &["one{# order}", "other{# orders}"]),
            ph("d", Some("date"), &["long"]),
        ],
    );
}

#[test]
fn parser_mixed_number_and_plural() {
    pretty_eq!(
        parse_template("Price: {price, number} ({qty, plural, one{# unit} other{# units}})"),
        vec![
            ph("price", Some("number"), &[]),
            ph("qty", Some("plural"), &["one{# unit}", "other{# units}"]),
        ],
    );
}

#[test]
fn parser_mixed_three_types() {
    pretty_eq!(
        parse_template("{name} paid {amount, number} on {date, date, short}"),
        vec![
            ph("name", None, &[]),
            ph("amount", Some("number"), &[]),
            ph("date", Some("date"), &["short"]),
        ],
    );
}

#[test]
fn parser_mixed_same_var_twice() {
    pretty_eq!(
        parse_template("User {user} logged in as {user}"),
        vec![ph("user", None, &[]), ph("user", None, &[])],
    );
}

#[test]
fn parser_mixed_some_vars_missing() {
    // Parser is structural, missing vs present vars produce the same output.
    pretty_eq!(
        parse_template("{name} owes {amount, number} by {due, date}"),
        vec![
            ph("name", None, &[]),
            ph("amount", Some("number"), &[]),
            ph("due", Some("date"), &[]),
        ],
    );
}

#[test]
fn parser_mixed_notification() {
    pretty_eq!(
        parse_template(
            "{actor} commented on your post from {date, date, long} ({n, plural, one{# like} other{# likes}})",
        ),
        vec![
            ph("actor", None, &[]),
            ph("date", Some("date"), &["long"]),
            ph("n", Some("plural"), &["one{# like}", "other{# likes}"]),
        ],
    );
}

#[test]
fn parser_mixed_time_and_number() {
    pretty_eq!(
        parse_template("Event at {t, time} - {seats, number} seats remaining"),
        vec![ph("t", Some("time"), &[]), ph("seats", Some("number"), &[])],
    );
}
