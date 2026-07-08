use crate::prelude::*;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Format an ICU `MessageFormat` template, substituting typed placeholders.
///
/// Supported forms:
///   {name}                                plain toString
///   {name, date}                          date (medium)
///   {name, date, short|long|full}         date with explicit style
///   {name, time}                          time-only
///   {amount, number}                      locale number
///   {count, plural, =0{..} one{..} other{..}}  plural, # = count.
///
/// `lookup(name)` supplies variable values, return None to preserve placeholder.
pub fn format_template(template: &str, lookup: &dyn Fn(&str) -> Option<IntlValue>, ctx: &IntlFormatter) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let bytes = template.as_bytes();
    let mut i = 0;

    while let Some(bi) = bytes.get(i) {
        if *bi != b'{' {
            out.push(*bi as char);
            i += 1;
            continue;
        }

        let start = i;
        let mut depth = 0usize;
        let mut j = i;
        while let Some(bj) = bytes.get(j) {
            match bj {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }

        if depth != 0 {
            out.push_str(&template[start..]);
            break;
        }

        let inner = &template[start + 1..j];
        let end = j;

        match parse_placeholder(inner) {
            Some((var, ph)) => match lookup(var) {
                Some(val) => format_slot(&mut out, &val, &ph, ctx),
                None => out.push_str(&template[start..=end]),
            },
            None => out.push_str(&template[start..=end]),
        }

        i = end + 1;
    }

    out
}

// ---------------------------------------------------------------------------
// Placeholder types
// ---------------------------------------------------------------------------

enum Ph<'a> {
    Raw,
    Date {
        style: &'a str,
    },
    Time,
    Number,
    Plural {
        cases: &'a str,
    },
}

fn parse_placeholder(inner: &str) -> Option<(&str, Ph<'_>)> {
    let (var, rest) = cut(inner, ',');

    if var.is_empty() || !var.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return None;
    }

    let ph = if rest.is_empty() {
        Ph::Raw
    } else {
        let (type_str, tail) = cut(rest, ',');
        match type_str {
            "date" => Ph::Date {
                style: tail,
            },
            "time" => Ph::Time,
            "number" => Ph::Number,
            "plural" => Ph::Plural {
                cases: tail,
            },
            _ => Ph::Raw,
        }
    };

    Some((var, ph))
}

fn cut(s: &str, sep: char) -> (&str, &str) {
    match s.find(sep) {
        None => (s.trim(), ""),
        Some(pos) => (s[..pos].trim(), s[pos + 1..].trim()),
    }
}

// ---------------------------------------------------------------------------
// Slot formatter
// ---------------------------------------------------------------------------

fn format_slot(out: &mut String, val: &IntlValue, ph: &Ph<'_>, ctx: &IntlFormatter) {
    match ph {
        Ph::Raw => out.push_str(&val.to_string()),

        Ph::Date {
            style,
        } => {
            let ts = val.as_int().unwrap_or(0);
            let key = match *style {
                "short" => "short_date",
                "long" | "full" => "long_date",
                _ => "date",
            };
            out.push_str(&(ctx.date)(ts, key));
        }

        Ph::Time => {
            let ts = val.as_int().unwrap_or(0);
            out.push_str(&(ctx.date)(ts, "time"));
        }

        Ph::Number => {
            let n = val.as_float().unwrap_or(0.0);
            out.push_str(&(ctx.number)(n));
        }

        Ph::Plural {
            cases,
        } => {
            let count = val.as_int().unwrap_or(0);
            let cat = (ctx.plural)(count);
            let exact = format!("={count}");
            let tpl = find_case(cases, &exact)
                .or_else(|| find_case(cases, cat))
                .or_else(|| find_case(cases, "other"));
            match tpl {
                Some(t) => out.push_str(&t.replace('#', &count.to_string())),
                None => out.push_str(&count.to_string()),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Template parser -- structural extraction (no formatting, no ICU)
// ---------------------------------------------------------------------------

/// Structural representation of one ICU placeholder extracted from a template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placeholder {
    /// The variable name, e.g. "name", "count", "due".
    pub var: String,
    /// The formatter type when present: "date", "time", "number", "plural", or
    /// any other string for unknown types.  None for raw `{name}` placeholders.
    pub fn_name: Option<String>,
    /// Extra arguments after the type:
    /// - date: [] or ["short"], ["long"], ["full"], ["custom"]
    /// - time: []
    /// - number: []
    /// - plural: each case as "keyword{body}", e.g. ["=0{zero}", "one{# item}", "other{# items}"]
    /// - raw (no type): []
    /// - unknown type: ["tail"] if a tail is present, otherwise []
    pub args: Vec<String>,
}

/// Parse all ICU placeholders from a template string.
///
/// Placeholders with invalid var names (empty, contains spaces, etc.) are
/// silently skipped, matching `format_template` behaviour.  Unclosed `{` stops
/// scanning from that point.
pub fn parse_template(template: &str) -> Vec<Placeholder> {
    let bytes = template.as_bytes();
    let mut result = Vec::new();
    let mut i = 0;

    while let Some(bi) = bytes.get(i) {
        if *bi != b'{' {
            i += 1;
            continue;
        }

        let start = i;
        let mut depth = 0usize;
        let mut j = i;
        while let Some(bj) = bytes.get(j) {
            match bj {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }

        if depth != 0 {
            break;
        }

        let inner = &template[start + 1..j];
        if let Some(ph) = parse_into_placeholder(inner) {
            result.push(ph);
        }

        i = j + 1;
    }

    result
}

fn parse_into_placeholder(inner: &str) -> Option<Placeholder> {
    let (var, rest) = cut(inner, ',');

    if var.is_empty() || !var.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return None;
    }

    if rest.is_empty() {
        return Some(Placeholder {
            var: var.to_owned(),
            fn_name: None,
            args: vec![],
        });
    }

    let (fn_name, tail) = cut(rest, ',');

    let args = if tail.is_empty() {
        vec![]
    } else if fn_name == "plural" {
        parse_plural_case_args(tail)
    } else {
        vec![tail.to_owned()]
    };

    Some(Placeholder {
        var: var.to_owned(),
        fn_name: Some(fn_name.to_owned()),
        args,
    })
}

fn parse_plural_case_args(cases: &str) -> Vec<String> {
    let bytes = cases.as_bytes();
    let mut result = Vec::new();
    let mut i = 0;

    while bytes.get(i).is_some() {
        while let Some(bi) = bytes.get(i)
            && bi.is_ascii_whitespace()
        {
            i += 1;
        }

        let kw_start = i;
        while let Some(bi) = bytes.get(i)
            && (bi.is_ascii_alphanumeric() || *bi == b'=' || *bi == b'-')
        {
            i += 1;
        }

        if kw_start == i {
            break;
        }
        let Some(&b'{') = bytes.get(i) else {
            break;
        };

        let mut depth = 0usize;
        let mut j = i;
        while let Some(bj) = bytes.get(j) {
            match bj {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }

        if depth != 0 {
            break;
        }

        result.push(cases[kw_start..=j].to_string());
        i = j + 1;
    }

    result
}

// ---------------------------------------------------------------------------
// Plural case scanner
// ---------------------------------------------------------------------------

pub fn find_case<'a>(cases: &'a str, keyword: &str) -> Option<&'a str> {
    let bytes = cases.as_bytes();
    let mut i = 0;

    while bytes.get(i).is_some() {
        while let Some(bi) = bytes.get(i)
            && bi.is_ascii_whitespace()
        {
            i += 1;
        }

        let kw_start = i;
        while let Some(bi) = bytes.get(i)
            && (bi.is_ascii_alphanumeric() || *bi == b'=' || *bi == b'-')
        {
            i += 1;
        }
        let kw = &cases[kw_start..i];

        let Some(&b'{') = bytes.get(i) else {
            break;
        };

        let content_start = i + 1;
        let mut depth = 0usize;
        let mut j = i;
        while let Some(bj) = bytes.get(j) {
            match bj {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }
        if depth != 0 {
            break;
        }

        if kw == keyword {
            return Some(&cases[content_start..j]);
        }

        i = j + 1;
    }

    None
}
