# Important

- Always use English in code and comments. Chat responses are in Vietnamese.
- Code need to be human readable, concise and focus. You can see the existing code base to know the current convention, follow exactly coding style, variable names, line break, block break, comma..
- Comments and docs comments:
  - Should not have semicolon, use comma instead.
  - Should not have backtick, we dont need styling code in comment.

- Do not use unwrap or expect or panic. Try to return result or something instead.
- All public items such as macro, function, struct, impl methods, trait, trait methods.. should come together with a docs comment explain about the item. But we dont need it in the ./tests folder, naming and convention should take care of them.
- Add section brief separator where needed, including tests:

```rs
// ---------------------------------------------------------------------------
// Section brief
// ---------------------------------------------------------------------------
```

- Use pretty_eq! macro for all assertions, including boolean. Always include an assertion description like: abc should xyz.
- See ./tests/independently.sh to find a coresponding command and run to test your change.
- Unit test names need to be meaningful semantic. Test data are all related to the Fringe TV series.

# Formatting Rules

## Characters

Use only ASCII (0x00-0x7F). Never use Unicode characters outside this range.

Banned examples:

| Category | Banned characters                |
| -------- | -------------------------------- |
| Dashes   | en dash, em dash, horizontal bar |
| Arrows   | any unicode arrows               |
| Quotes   | smart quotes, curly apostrophes  |
| Bullets  | bullet, triangle, diamond        |
| Math     | multiplication, division, minus  |
| Emoji    | all emoji without exception      |
| Misc     | ellipsis, checkmark, copyright   |

Use instead:

- Dashes: plain hyphen-minus (-)
- Arrows: -> or <- or => or <= (two ASCII chars)
- Quotes: straight double quotes (") or straight single quotes (')
- Bullets: plain hyphen (-) or asterisk (\*) or plus (+)
- Math: use plain ASCII operators (\*, /, -)
- Ellipsis: two plain periods (..)
- Copyright: (c)
- Semicolon: colon (,)

## Formatting

- Use plain Markdown only: headings (#), bold (\*_), italic (_), code fences (```), tables, blockquotes (>).
- No decorative Unicode borders, box-drawing characters, or special symbols.
- Code blocks should use ASCII-only content unless quoting external source verbatim.

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
<!-- END doctoc generated TOC please keep comment here to allow auto update -->
