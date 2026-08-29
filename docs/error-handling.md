# Error handling

```rs
#[grand_line_err]
enum MyErr {
    #[error("record not found")]
    #[client] // forwarded to the response as-is
    NotFound,

    #[error("oops")] // client sees a generic "internal server error"
    InternalProblem,
}

// Raise from any resolver:
Err(MyErr::NotFound)?;

// Downcast from a response error:
error.source
    .as_deref()
    .and_then(|e| e.downcast_ref::<GrandLineErr>())
    .map(|e| e.0.code()); // e.g. "NotFound"
```

Any error that isn't `#[client]` is logged to stderr with its real message and replaced with a generic internal-server error before it reaches the client, so accidental leaks of internal detail are opt-in, not opt-out.

Each package that ships a `#[grand_line_err]` enum exposes it under a package-specific alias in `grand_line::prelude` rather than the bare `MyErr` name (every package names its own internal enum `MyErr`, so bare `MyErr` would collide across crates). The aliases actually in scope today:

| Package          | Alias       |
| ---------------- | ----------- |
| `crates/core/db` | `CoreDbErr` |
| `crates/http`    | `HttpErr`   |
| `crates/auth`    | `AuthErr`   |
| `crates/authz`   | `AuthzErr`  |

Your own app's `#[grand_line_err]` enum keeps whatever name you give it (e.g. `SaasErr` in the [saas example](https://github.com/namnm/grand-line/blob/master/examples/saas/src/err.rs)) - the aliasing convention above is specific to how the framework's own internal crates avoid colliding with each other and with yours.
