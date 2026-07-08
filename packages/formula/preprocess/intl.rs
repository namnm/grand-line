use std::borrow::Cow;

use sourcemap::{SourceMap, SourceMapBuilder};

use super::CodeScanner;

// ---------------------------------------------------------------------------
// preprocess_intl_template
// ---------------------------------------------------------------------------
//
// Transforms `intl`...`` tagged templates into `intl("...", #{var: var})` calls.
//
// The tagged template content (between backticks) is an ICU MessageFormat string.
// Top-level `{varName}` and `{varName, type, ...}` placeholders are extracted,
// the corresponding Rhai identifiers are passed as a map literal.
//
// Generated forms:
//   intl`No vars` -> intl("No vars")
//   intl`Hello {name}!` -> intl("Hello {name}!", #{name: name})
//   intl`{a} and {b}` -> intl("{a} and {b}", #{a: a, b: b})
//
// Rules:
//   - The character before `intl` must not be alnum or `_` (word boundary).
//   - `intl`` inside string literals or comments is NOT transformed.
//   - `\`` inside the template is an escaped backtick, becomes literal `` ` `` in
//     the generated string literal (the `\` is dropped).
//   - Duplicate ICU var names produce a single entry in the map.
//   - An unclosed backtick is left untouched (no partial replacement).

/// Preprocess a Rhai script, transforming `intl`...`` tagged templates.
/// Returns the possibly-transformed script (borrowed when unchanged).
pub fn preprocess_intl_template(script: &str) -> Cow<'_, str> {
    preprocess_intl_template_with_map(script).0
}

/// Like `preprocess_intl_template`, but also returns a Source Map v3 object
/// mapping positions in the generated output back to the original script.
///
/// The source map is `None` when no transformation occurred (the output is
/// identical to the input, so positions are already correct).
/// When `Some`, use `SourceMap::lookup_token(line, col)` with 0-indexed
/// coordinates to translate a generated position to the original source.
pub fn preprocess_intl_template_with_map(script: &str) -> (Cow<'_, str>, Option<SourceMap>) {
    let bytes = script.as_bytes();
    let mut scanner = CodeScanner::new(bytes);
    let mut result: Option<String> = None;
    let mut builder: Option<SourceMapBuilder> = None;
    let mut last_copy = 0usize;
    let mut out_pos = Pos::default();
    let mut in_pos = Pos::default();

    while !scanner.done() {
        if !scanner.in_code() {
            scanner.step();
            continue;
        }

        let pos = scanner.pos;

        // Detect `intl`` in code context: 4-byte keyword followed by backtick.
        if bytes.get(pos) == Some(&b'i')
            && bytes.get(pos + 1) == Some(&b'n')
            && bytes.get(pos + 2) == Some(&b't')
            && bytes.get(pos + 3) == Some(&b'l')
            && bytes.get(pos + 4) == Some(&b'`')
        {
            // Word-boundary check: char before 'i' must not be alnum or '_'.
            let preceded_by_ident = if pos > 0
                && let Some(b) = bytes.get(pos - 1)
            {
                b.is_ascii_alphanumeric() || *b == b'_'
            } else {
                false
            };

            if !preceded_by_ident {
                let tpl_start = pos + 5; // first byte after the opening backtick
                let (tpl, end) = extract_backtick_body(bytes, tpl_start);

                if let Some(end_pos) = end {
                    let out = result.get_or_insert_with(|| String::with_capacity(script.len() + 64));
                    let b = builder.get_or_insert_with(|| SourceMapBuilder::new(None));

                    // Flush unchanged segment [last_copy..pos] with per-line mappings.
                    let segment = &script[last_copy..pos];
                    flush_segment(b, segment, out, &mut out_pos, &mut in_pos);

                    // Map the start of the generated intl(...) call -> intl` in input.
                    b.add(out_pos.line, out_pos.col, in_pos.line, in_pos.col, None, None, false);

                    // Emit the generated call and advance the output column.
                    let before = out.len();
                    emit_intl_call(out, &tpl);
                    // Generated intl(...) is always single-line.
                    let added_cols = out[before..].chars().count() as u32;
                    out_pos.col += added_cols;

                    // Advance in_pos past the original intl`...` span.
                    advance_pos(&mut in_pos, &script[pos..end_pos]);

                    last_copy = end_pos;
                    scanner.pos = end_pos;
                    continue;
                }
            }
        }

        scanner.step();
    }

    match result {
        None => (Cow::Borrowed(script), None),
        Some(mut s) => {
            // Flush the final unchanged tail.
            let remaining = &script[last_copy..];
            if let Some(b) = builder.as_mut() {
                flush_segment(b, remaining, &mut s, &mut out_pos, &mut in_pos);
            } else {
                s.push_str(remaining);
            }
            let sm = builder.map(|b| b.into_sourcemap());
            (Cow::Owned(s), sm)
        }
    }
}

// ---------------------------------------------------------------------------
// Position tracking helpers
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, Default)]
struct Pos {
    line: u32,
    col: u32,
}

// Append an unchanged segment to `out` while recording per-line source map
// mappings. Both `out_pos` and `in_pos` advance by the same amount through
// unchanged text; they only diverge at transformation boundaries.
fn flush_segment(builder: &mut SourceMapBuilder, segment: &str, out: &mut String, out_pos: &mut Pos, in_pos: &mut Pos) {
    if segment.is_empty() {
        return;
    }
    builder.add(out_pos.line, out_pos.col, in_pos.line, in_pos.col, None, None, false);
    let mut rest = segment;
    while let Some(nl) = rest.find('\n') {
        let cols = rest[..nl].chars().count() as u32;
        out_pos.col += cols;
        in_pos.col += cols;
        out_pos.line += 1;
        out_pos.col = 0;
        in_pos.line += 1;
        in_pos.col = 0;
        builder.add(out_pos.line, out_pos.col, in_pos.line, in_pos.col, None, None, false);
        rest = &rest[nl + 1..];
    }
    let cols = rest.chars().count() as u32;
    out_pos.col += cols;
    in_pos.col += cols;
    out.push_str(segment);
}

// Advance a position counter through an arbitrary string (used for the
// original intl`...` span that is replaced in the output).
fn advance_pos(pos: &mut Pos, s: &str) {
    for ch in s.chars() {
        if ch == '\n' {
            pos.line += 1;
            pos.col = 0;
        } else {
            pos.col += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Template body extraction and call emission
// ---------------------------------------------------------------------------

// Collect template content starting at `start`, up to the unescaped closing backtick.
// `\`` -> literal `` ` `` (escaped backtick inside a template).
// Returns (content_string, Some(pos_after_closing_backtick)) or (_, None) if unclosed.
fn extract_backtick_body(bytes: &[u8], start: usize) -> (String, Option<usize>) {
    let mut i = start;
    let mut content = String::new();

    while let Some(bi) = bytes.get(i) {
        if *bi == b'\\' && bytes.get(i + 1) == Some(&b'`') {
            content.push('`');
            i += 2;
        } else if *bi == b'`' {
            return (content, Some(i + 1));
        } else {
            content.push(*bi as char);
            i += 1;
        }
    }

    (content, None) // unclosed
}

// Write `intl("escaped_tpl")` or `intl("escaped_tpl", #{v1: v1, v2: v2})`.
fn emit_intl_call(out: &mut String, template: &str) {
    let vars = extract_icu_vars(template);
    let escaped = escape_for_double_quote(template);

    out.push_str("intl(\"");
    out.push_str(&escaped);
    out.push('"');

    if !vars.is_empty() {
        out.push_str(", #{");
        for (i, var) in vars.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(var);
            out.push_str(": ");
            out.push_str(var);
        }
        out.push('}');
    }

    out.push(')');
}

// ---------------------------------------------------------------------------
// Helpers (also used by lib.rs)
// ---------------------------------------------------------------------------

// Extract top-level ICU placeholder variable names from a template string.
// For `{varName}` and `{varName, type, ...}` returns `varName`.
// Nested braces (ICU plural/select sub-patterns) are skipped.
// Result is ordered and deduplicated.
pub fn extract_icu_vars(template: &str) -> Vec<String> {
    let bytes = template.as_bytes();
    let mut vars: Vec<String> = Vec::new();
    let mut i = 0;
    let mut depth = 0usize;

    while let Some(bi) = bytes.get(i) {
        if *bi == b'{' {
            depth += 1;
            if depth == 1 {
                let mut j = i + 1;
                while let Some(bj) = bytes.get(j)
                    && *bj == b' '
                {
                    j += 1;
                }
                let name_start = j;
                while let Some(bj) = bytes.get(j)
                    && (bj.is_ascii_alphanumeric() || *bj == b'_')
                {
                    j += 1;
                }
                if j > name_start {
                    let name = template[name_start..j].to_string();
                    if !vars.contains(&name) {
                        vars.push(name);
                    }
                }
            }
        } else if *bi == b'}' && depth > 0 {
            depth -= 1;
        }
        i += 1;
    }
    vars
}

// Escape a string for use as a Rhai double-quoted string literal.
pub fn escape_for_double_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}
