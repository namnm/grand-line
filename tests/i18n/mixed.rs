pub use grand_line::prelude::*;

#[path = "./prelude.rs"]
mod prelude;
use prelude::*;

// ---------------------------------------------------------------------------
// Existing tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mixed_member_since_en() -> Res<()> {
    let tpl = "Member since {joined, date, short}";
    let v = [("joined", IntlValue::Int(JAN_15_2024))];
    intl(tpl, &v, "Member since 1/15/24")
}

#[tokio::test]
async fn mixed_invoice_en() -> Res<()> {
    let tpl = "Invoice #{id} - Total: {amount, number} - Due: {due, date, long}";
    let v = [
        ("id", IntlValue::Int(42)),
        ("amount", IntlValue::Float(1_299.99)),
        ("due", IntlValue::Int(JAN_15_2024)),
    ];
    intl(tpl, &v, "Invoice #42 - Total: 1,299.99 - Due: January 15, 2024")
}

// ---------------------------------------------------------------------------
// New multi-type templates
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mixed_greeting_with_unread_count() -> Res<()> {
    let tpl = "Hello {name}! You have {n, plural, one{# message} other{# messages}}.";
    let v = [("name", IntlValue::Str("Alice".into())), ("n", IntlValue::Int(3))];
    intl(tpl, &v, "Hello Alice! You have 3 messages.")
}

#[tokio::test]
async fn mixed_plural_and_date() -> Res<()> {
    let tpl = "{n, plural, one{# order} other{# orders}} since {d, date, long}";
    let v = [("n", IntlValue::Int(1)), ("d", IntlValue::Int(JAN_15_2024))];
    intl(tpl, &v, "1 order since January 15, 2024")
}

#[tokio::test]
async fn mixed_number_and_plural() -> Res<()> {
    let tpl = "Price: {price, number} ({qty, plural, one{# unit} other{# units}})";
    let v = [("price", IntlValue::Float(49.99)), ("qty", IntlValue::Int(2))];
    intl(tpl, &v, "Price: 49.99 (2 units)")
}

#[tokio::test]
async fn mixed_three_types() -> Res<()> {
    let tpl = "{name} paid {amount, number} on {date, date, short}";
    let v = [
        ("name", IntlValue::Str("Bob".into())),
        ("amount", IntlValue::Float(250.0)),
        ("date", IntlValue::Int(JAN_15_2024)),
    ];
    intl(tpl, &v, "Bob paid 250 on 1/15/24")
}

#[tokio::test]
async fn mixed_same_var_referenced_twice() -> Res<()> {
    // The same variable appears twice, both occurrences are substituted.
    let tpl = "User {user} logged in as {user}";
    let v = [("user", IntlValue::Str("admin".into()))];
    intl(tpl, &v, "User admin logged in as admin")
}

#[tokio::test]
async fn mixed_some_vars_missing_others_present() -> Res<()> {
    // Placeholders for missing vars are preserved, present vars are formatted.
    let tpl = "{name} owes {amount, number} by {due, date}";
    let v = [("name", IntlValue::Str("Carol".into()))];
    // "amount" and "due" are not provided -> preserved as-is
    intl(tpl, &v, "Carol owes {amount, number} by {due, date}")
}

#[tokio::test]
async fn mixed_notification_template() -> Res<()> {
    let tpl = "{actor} commented on your post from {date, date, long} ({n, plural, one{# like} other{# likes}})";
    let v = [
        ("actor", IntlValue::Str("Dave".into())),
        ("date", IntlValue::Int(JAN_15_2024)),
        ("n", IntlValue::Int(42)),
    ];
    intl(tpl, &v, "Dave commented on your post from January 15, 2024 (42 likes)")
}

#[tokio::test]
async fn mixed_time_and_number() -> Res<()> {
    let tpl = "Event at {t, time} - {seats, number} seats remaining";
    let v = [
        ("t", IntlValue::Int(JAN_15_2024_1430)),
        ("seats", IntlValue::Int(5_432)),
    ];
    let e = &"Event at 2:30:00\u{202f}PM - 5,432 seats remaining".to_owned();
    intl(tpl, &v, e)
}
