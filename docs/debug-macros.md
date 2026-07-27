# Debug macro outputs

When a macro-generated resolver/model doesn't compile, or behaves unexpectedly, the compiler error points into the macro expansion, not your source (see [Design notes](contribution/design-notes.md#known-limitations)). Set `DEBUG_MACRO=1` and enable one of these feature flags to see the actual generated code:

- `debug_macro_cli` - prints generated code to stdout during build
- `debug_macro_file` - writes generated code to `target/grand-line/` during build

```toml
grand-line = { path = "...", features = ["debug_macro_file"] }
```

```sh
DEBUG_MACRO=1 cargo build
```

`debug_macro_file` writes one file per macro invocation to `target/grand-line/<name>.rs` (`<name>` is the model/resolver the macro was applied to), running `rustfmt` on it automatically - open the one matching what you're debugging and read the expanded Rust directly. `debug_macro_cli` prints the same generated code to stdout instead, with syntax highlighting via `prettyplease`.
