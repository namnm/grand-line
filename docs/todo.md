# todo

Open issues found while reworking resolver guards, model di macros, and reviewing
the transaction lifecycle. Ordered by priority. Items marked `[file_upload]` only
exist on that branch and block merging it into master.

---

## P0 - blocks a merge, or loses and leaks data silently

### 1. `has_actor` fix lives only on the file_upload merge commit

**Why.** Master's crud macros pick the ctx aware db helpers purely from
`cfg!(feature = "auth")` in [`utils/auth.rs`](../crates/macro_proc/utils/auth.rs).
That cfg reflects whether the proc macro package was built with the auth feature,
which cargo unifies across the whole workspace, not whether the package being
compiled can see the auth traits. Any workspace package that uses `#[create]` /
`#[update]` / `#[delete]` without depending on `_auth` gets `exec(ctx)` and
`ctx.auth()` generated into it and fails to compile. Master has no such package
today so nothing catches it, but `_file` on the file_upload branch does, and that
is how it was found.

**Fix.** Cherry-pick the fix already made on the file_upload merge commit: restore
a per resolver signal, `ResolverTyAttr::has_actor()` returning
`cfg!(feature = "auth") && !self.check.is_empty()`, and pass it into
`gen_am_exec` / `gen_am_into` / `gen_auth_by_id`. A resolver with no `check`
declares no dependency on auth and gets the plain helpers. This is safe now that
`IntoAmCtx` fills audit fields gracefully, so a `check = unauthenticated` resolver
simply leaves them unset instead of erroring.

### 2. `[file_upload]` `file` and `authz` cannot be compiled together

**Why.** [`gen_authz_row`](../crates/macro_proc/utils/authz.rs) emits
`ctx.authz_row_graceful::<F>()` into every crud resolver whenever the `authz` and
`resolver_authz_row` features are on. `_file` depends only on `_core`, so the
method is not in scope and `crates/file/resolvers/file_crud.rs` and
`file_delete.rs` fail to compile. `tests/independently.sh` runs the file suite
with `test_utils,sqlite,file` only, which hides it, but `make test` (which uses
the default feature set, authz included) goes red.

**Fix.** Same shape as item 1, the `has_actor` signal: make `authz_row` default to off when the resolver
declares no `check`, rather than keying purely off a global feature. A resolver
that never runs an authz guard has no row policy to apply anyway. The quick
alternative is spelling `authz_row = false` on the four `_file` resolvers, but
that leaves the same trap for the next package.

### 3. `[file_upload]` every file resolver is unauthenticated and cannot be guarded

**Why.** All seven operations the `_file` package ships are public. `fileSearch`
and `fileDetail` return `downloadUrl`, a presigned GET url for any file, to any
caller. `fileDelete(permanent: true)` removes objects from the bucket.
`fileCleanupExpiredPending` deletes rows in bulk. `fileUploadInit` hands out
presigned PUT urls with no size bound. The host cannot fix this: the package
ships built resolvers plus `FileMergedQuery` / `FileMergedMutation`, so there is
nowhere to attach a `check`. Dropping a resolver from the merge is possible but
then the capability is gone entirely, since the logic lives in the resolver body
and is not exported as a callable function.
[`docs/file-upload.md`](file-upload.md) acknowledges the gap and suggests wrapping
with `authz(realm = "system")`, which is not something the host can do to a
resolver it does not own (and that attribute no longer exists).

**Fix.** Add a runtime hook to `FileHandlers`, consistent with how the package
already exposes `key` and `on_upload_confirm`, so `_file` still does not need to
depend on `_authz`:

```rs
pub enum FileOp {
    Read,
    Init,
    Confirm,
    Delete,
    DeletePermanent,
    Cleanup,
}

async fn authorize(&self, ctx: &Context<'_>, op: FileOp) -> Res<()>;
```

Each resolver calls `c.handlers.authorize(ctx, op).await?` first. Decide the
default deliberately: `Ok(())` matches the package's existing permissive defaults,
but `DeletePermanent` and `Cleanup` are destructive enough to warrant deny by
default. Whatever is chosen belongs in the Setup section of the doc, not buried
further down.

---

### 4. `History` has no org column, so a row policy cannot scope it

**Why.** Item 4's original half (secrets landing in `History.data`) is fixed, see
the Fixed section. What remains is the row boundary. `History` carries
`entity_type`, `entity_id`, `by_id` and timestamps for every audited model in one
shared table, and has no column an authz row policy can filter on. A
`#[search(History)]` is therefore all-or-nothing: whoever passes the guard sees the
audit metadata of every model and every org, even with `data` redacted.

**Fix.** Give `History` an optional org column that the owning model can populate,
so a row policy has something to filter on. `_core` cannot depend on `_authz`, so
the mapping has to be declared by the model rather than assumed by the framework,
e.g. reusing the `#[authz_org_id]` signal already used for org scoping. Until then,
the docs say not to expose `History` behind a guard weaker than the strongest model
being audited.

## P1 - correctness bugs, latent but severe when hit

### 5. A subscription keeps the permissions it opened with

**Why.** Guards and the resolver body run once, when the client subscribes. The
per request cache holding the matched role, its col and row policy, and the
resolved session then lives as long as the subscription does. Revoking a role,
editing a policy, or deleting a session has no effect on a stream that is already
open. The row filter is still enforced on every event, but that filter itself was
computed under the old permissions.

A lifetime bound on the subscription only narrows the window, it never closes it,
so it is a backstop and not the mechanism.

**Fix: a version counter, read at delivery time.** Keep a monotonically increasing
counter per principal, bumped by anything that changes an authorization: a role,
a user to role assignment, a col or row policy, a session. Store it where every
instance can read it, redis when the app is scaled out, a small table otherwise.
Before delivering an event, read the counter (one cheap lookup) and re-run the
guard only when it moved.

Exact, and cheap in the common case: the full guard only runs when something
actually changed. It is a pull rather than a push, so it does not depend on an
invalidation message being delivered, which is what makes it correct rather than
merely fast.

The alternatives, for the record:

- Re-run the guard on every event. Exact, but three queries per event per
  subscriber. Worth having as a config option for an app that does not want to
  run a shared counter store, not as the default.
- Push invalidation over the subscription broker itself. `#[authz_role]`,
  `#[authz_user_in_role]` and `#[auth_session]` already declare which models
  those are, and their crud mutations already publish, so a subscription could
  listen for changes to the identity it was authorized under. Zero cost per data
  event and millisecond latency, but a dropped message silently returns to stale.
  Best used alongside the counter, to cut latency, not instead of it.
- Express the row policy as a join instead of a materialized condition, so
  `WHERE org_id = 'X'` becomes `WHERE org_id IN (SELECT ..)`. A revoked
  assignment then stops matching inside the reload query that already runs, at no
  extra cost, and exactly. It only covers the row boundary, not a changed
  col_policy or a deleted session, and needs the row policy filter shape to be
  able to carry a subquery. Worth doing where it applies, on top of the counter.

**The counter is not only for authz.** Anything that caches a decision for longer
than one request has the same problem, and the same counter answers it: a
subscription's `ctx.cache()` entries, any process level cache of policy or config
an app adds later, invalidation across instances without trusting message
delivery, and a detached job (item 12, the detached job helper) that started before a change landed and
should notice before it writes. Design it as a general resource version, keyed by
principal and by resource, not as an authz specific hack.

### 6. `AuthOtpImpl`'s signature lets a consumer lose the attempt increment

**Why.** `auth_otp_ensure_resolve` increments the attempt counter and then returns
`OtpResolveInvalid` on a bad code. That error rolls the request transaction back.
If a consumer's `increment` runs on `ctx.tx()` instead of `ctx.db()`, the
increment is rolled back with it, the counter never rises, and OTP brute forcing
becomes unlimited. The framework's own `DefaultOtpImpl` uses `ctx.db()` and
[`tests/auth/otp.rs`](../tests/auth/otp.rs) covers it, so this is not a live bug,
but nothing stops a consumer from getting it wrong: the DI trait hands them `ctx`,
and the hand written example in [authentication.md](authentication.md) shows
`AuthSessionImpl` using `ctx.tx()` right above an `AuthOtpImpl` skeleton.

**Fix.** Change the trait to hand out the connection instead of the context, so
the mistake is not expressible:

```rs
async fn increment(&self, db: &DatabaseConnection, id: &str, ty: &str) -> Res<Option<AuthImplOtp>>;
```

Same for `find` / `reset` / `delete`. Cheaper fallback if the signature change is
not wanted: state the requirement and the reason in the doc next to the skeleton.

### 7. `auth_otp_ensure_re_request` deletes on db and re-creates on tx

**Why.** [`otp_context.rs`](../crates/auth/context/otp_context.rs) deletes the
stale otp row on the raw connection, so it commits immediately, while the caller
then creates the replacement row on the request transaction. If anything later in
the resolver fails (a mailer call, for instance), the delete stands and the create
is rolled back: the user loses their pending otp entirely and eats the cooldown
before they can request another.

**Fix.** Either move the delete onto the request transaction and accept that a
failed request keeps the stale row (the cooldown check already tolerates that), or
defer the delete until after the new row is written. The first is simpler and
matches what the cooldown is for.

### 8. A `#[resolver]` returning an object type still under-selects silently

**Why.** The scalar case is fixed, see the Fixed section: `gql_load` now errors
when the calling field has no selection set at all. That signal does not cover a
`#[resolver]` field whose return type is an object. There the selection set is not
empty, it just names fields of a different type, so nothing maps through
`Self::gql_select()`, the lookahead comes back empty, and the loaded row is all
`None` again.

It cannot be told apart from the legitimate empty-lookahead case by shape alone:
`{ user { __typename } }` and a `#[resolver]` field declaring no `sql_dep` both
produce an empty lookahead at a position that really is describing the entity.

**Fix.** Probably none beyond what is there. `gql_load_with` covers it, and the
doc says to use it whenever the calling field's selection set does not describe
the loaded entity. Closing it properly would need the lookahead to know which gql
type the current position returns, which async-graphql's `SelectionField` does not
expose. Keep as a known limitation unless it is reported again.

---

### 9. The session cookie sets no `SameSite` and no `Path`

**Why.** `HttpContext::set_cookie` sets `http_only` and `secure` and stops there.
Two things follow, neither of them visible until something breaks:

- With no `SameSite` attribute browsers apply `Lax`, so a browser app on a
  different origin than the API simply never sends the session cookie, and the
  request looks unauthenticated with no error explaining why. Such a setup needs
  `SameSite=None; Secure`, which the framework has no way to express.
- With no `Path` the browser derives one from the request URI, so a cookie set
  from `/api/graphql` is scoped to `/api` and is not sent anywhere else.

`SameSite` is also the main CSRF control for a cookie-authenticated API, so
leaving it implicit is a security decision made by accident.

**Fix.** Put both on `AuthConfig` next to the existing cookie key and expiry, with
`SameSite=Lax` and `Path=/` as explicit defaults, and say in
[authentication.md](authentication.md) which value a cross-origin app needs.

## P2 - robustness and missing patterns

### 10. No supported way to keep working after the mutation returns

**Why.** The per request transaction is held for the whole resolver body, so a
mutation that kicks off slow work (a solver subprocess, a transcode) either holds
a connection open for the duration or goes around the framework. Both have already
happened: a real app hand rolled `tokio::spawn` plus a second transaction, and
`FfmpegFileHandlers::on_upload_confirm` on the file_upload branch spawns a task
that races the request transaction - on a fast path it can `UPDATE` the row before
the request commits (postgres blocks until commit, sqlite can return
`database is locked`), and if the request rolls back afterwards the task still
writes `Ready`.

**Fix.** An official detached job queue on `GrandLineData`:

```rs
detached: Mutex<Vec<BoxFuture<'static, ()>>>,

async fn detach<F, Fu>(&self, f: F) -> Res<()>
where
    F: FnOnce(Arc<DatabaseConnection>) -> Fu + Send + 'static,
    Fu: Future<Output = Res<()>> + Send + 'static;
```

`cleanup` commits first and only spawns the queued jobs on a successful commit; a
rollback drops the queue, which is the right semantics - do not run background work
for a request that did not land. Handing the closure an `Arc<DatabaseConnection>`
also makes it impossible for a detached job to capture the request transaction, so
this cannot reintroduce the commit ownership bug fixed earlier this session.

### 11. `grand_line_build` reports errors in a way cargo ignores

**Why.** [`grand_line_build/lib.rs`](../crates/grand_line_build/lib.rs) reports
every failure with `eprintln!("cargo:error=...")`. Cargo reads build script
directives from stdout, not stderr, and `cargo:error=` with a single colon is not
a directive at all (the build-failing form is `cargo::error=`, added in Rust 1.84).
So a missing `CARGO_MANIFEST_DIR`, a missing `OUT_DIR`, and a failed write of
`grand_line_schema.rs` all print into a stream nobody reads and let the build pass.

On top of that, `scan(path)` silently scans nothing when the path does not exist.
A real refactor left several stale `.scan("../../crates/commerce/...")` entries
pointing at deleted directories and `cargo check` stayed green.

**Fix.** Use `panic!` for all of these - cargo surfaces a build script panic and
fails the build. Add an `is_dir()` check per configured scan dir with a message
naming the path.

### 12. `AuthOtpContext` increments before verifying the secret

**Why.** [`otp_context.rs`](../crates/auth/context/otp_context.rs) calls
`increment` before checking the code or the secret. The otp row id is handed to
the client by `register`, so anyone holding an id can burn a legitimate user's
attempts and lock them out.

**Fix.** Verify the secret first and only then consume an attempt. Keep the single
opaque `OtpResolveInvalid` for every failure path so this does not become an
oracle for which part was wrong.

### 13. Dropping a transaction while its connection is locked panics

**Why.** sea-orm 2.0.0's `Drop for DatabaseTransaction` calls
`start_rollback().expect(..)`, and `start_rollback` returns
`Err("Dropping a locked Transaction")` when `conn.try_lock()` fails. Any code path
that drops a transaction while another task holds the connection lock panics.

`ConnX` removed every framework path that could get there: the transaction is
owned by `GrandLineData`, borrowed per statement, and never handed out, so nothing
can be holding it when it is dropped. What remains is `tx_release`, which drops it
from a `Drop` impl after a `try_lock`, and only when a subscription stream is torn
down mid statement.

**Fix.** Low priority now, but `tx_release` could await the lock from a spawned
task instead of dropping under `try_lock`, which would close the last window.

### 14. Opening the transaction holds its mutex across `db.begin()`

**Why.** `GrandLineData::tx_begin` holds the `tx` mutex for the whole
`db.begin().await`. When the pool is exhausted every sibling resolver asking for a
connection on a mutation queues behind it, and the symptom from outside is an
unexplained hanging request. Queries are unaffected, they never call it.

**Fix.** Not incorrect, so low priority, but worth either releasing the lock around
`begin` (accepting that two racing callers may open one transaction too many and
discarding the loser) or documenting the behavior so the queue is recognizable when
diagnosing.

### 15. No timeout bounds the request transaction

**Why.** `cleanup` only runs once `next.run()` returns, so a resolver making an
external call with no timeout holds the transaction and its connection open
indefinitely. This is inherent to one transaction per request, not a bug, but
nothing in the framework or the docs bounds it.

**Fix.** Document the recommendation to set `statement_timeout` and
`idle_in_transaction_session_timeout` on the database side.

### 16. `[file_upload]` hardening for the file package

**Why and fix**, each small and independent:

- Rows stuck in `Processing` have no reaper. `Pending` has
  `fileCleanupExpiredPending`, but a task that panics or a process restart leaves
  `Processing` forever. Add an equivalent sweep, or a timestamp plus a status
  transition the sweep can act on.
- `filename` comes from the client and flows straight into the S3 object key
  (`{ulid}/{filename}`) and into temp file names passed to ffmpeg. Not path
  traversal (S3 keys are opaque and `Path::extension` cannot contain a separator)
  but unbounded and unsanitized. Clamp the length and strip control characters.
- `fileUploadInit` accepts `org_id` from the client without verifying it. Harmless
  while everything is unauthenticated, a hole the moment item 3 lands.
- `fileUploadConfirm` reads the real object size from `HeadObject` but enforces no
  maximum, and does not check `uploadExpiresAt`. Add both.
- `minify_image` passes `-q:v` (an mjpeg option) while naming the output after the
  original filename's extension, so png and webp silently ignore quality, and an
  unusual extension makes ffmpeg fail to pick a muxer and marks the row `Failed`.
  Pick the output container explicitly.

---

### 17. A client can ask for an unbounded `offset`

**Why.** `CoreConfig` clamps `limit` through `limit_max`, but `Pagination::inner`
passes `offset` straight through untouched. `offset: 100000000` against a large
table is a full scan the database cannot shortcut, from a single ordinary looking
query. `order_by` has the same shape of problem, a client may send an arbitrarily
long `Vec<OrderBy>`, though the database usually rejects that first.

**Fix.** An `offset_max` on `CoreConfig` alongside `limit_max`, clamped the same
way, and a cap on the number of `order_by` entries accepted.

### 18. `get_ip` trusts client supplied headers

**Why.** `HttpContext::get_ip` reads `x-real-ip`, then `x-forwarded-for`, then
`x-socket-addr`, with no notion of which of those a proxy is actually setting.
Behind a proxy that overwrites them this is right; reachable directly, or behind a
proxy that appends rather than replaces, any client picks its own address. The
saas example persists that value on the login session as audit data, so what looks
like a record of where a session came from is client controlled.

**Fix.** Make the source configurable rather than a fixed fallback chain: let the
app say which header its proxy sets and how many hops to trust, and fall back to
the socket address when nothing is configured. Until then, say plainly in the docs
that the header chain is only trustworthy behind a proxy that overwrites it.

### 19. A missing `User-Agent` header blocks login

**Why.** `get_ua` returns `HeaderUa404` when the `user-agent` header is absent, and
the saas login path calls `ctx.get_ua()?` to record the session. A programmatic
client that sends no `User-Agent`, which is neither unusual nor wrong, cannot log
in at all. A purely informational field is a hard failure.

**Fix.** Let `get_ua` return what it found and leave the decision to the caller, or
keep a strict variant and have the session helper use the lenient one. Recording
an empty user agent is strictly better than refusing the login.

### 20. `arithmetic_side_effects` is allowed workspace wide

**Why.** The workspace lints switch the whole clippy restriction group on and then
allow `arithmetic_side_effects` globally. Release builds wrap silently on overflow,
and the codebase does have hand written arithmetic on untrusted sizes, the depth
counter in the i18n template parser among them. That particular one is safe, its
counter is always at least one when it is decremented, but the guarantee comes
from reading the loop rather than from the compiler.

**Fix.** Drop the global allow and put a narrow `#[allow]` with a reason at the
handful of sites that need it, which is also what makes the safe ones self
documenting.

## P3 - documentation

### 21. Mixing `ctx.db_pool()` and the request connection is only half documented

**Why.** [resolvers.md](resolvers.md#connections-and-transactions) now names the
three handles and says what each is for, but not what it costs to mix them. On
sqlite, writing through `ctx.db_pool()` while the request transaction already
holds a write lock returns `SQLITE_BUSY`. The otp flow happens to be safe because
its writes run before the transaction takes a lock, which is an accident of
ordering rather than a guarantee, and nothing says so.

**Fix.** Add the locking caveat next to the handle table, and say plainly that
anything written through `ctx.db_pool()` is outside the request's rollback, which
is the whole point of it and also the whole risk.

### 23. Close the `From<async_graphql::Error> for GrandLineErr` request

**Why not.** The case that prompted it was reaching for the raw db handle, and
`ctx.db().await?` already covers that. For the general case,
`ctx.data_opt_impl::<T>()` returns an `Option`, so `.ok_or(MyErr)?` works with no
conversion. A blanket `From` would be actively harmful: `async_graphql::Error` is
an opaque client-facing type, and auto-converting it invites internal messages to
leak past the framework's deliberately opt-in `#[client]` error model.

**Fix.** Document `ctx.db()` and `data_opt_impl` in [resolvers.md](resolvers.md)
instead, and drop the request.

---

## Open decision, not yet an item

After a normal rollback (a resolver errored, `cleanup` succeeded), the response
still carries `data` produced by mutations that were rolled back. A client reading
only `data` sees rows that do not exist. Nulling `data` on any rollback would be
consistent with one transaction per request, but it conflicts with GraphQL's
partial success semantics and is harmless for queries, where a rollback discards
nothing. Needs a decision before it becomes an item.

---

## Fixed in this session

Recorded so they are not re-reported.

- **The dataloader cache key was unstable, so batching silently fell back to n+1.**
  Found while working item 8, and the more severe half of it. `gql_look_ahead_of`
  collected through a `HashSet` and iterated it, and `ColumnX::to_loader_key`
  writes that order into the key (item 8's claim that the lookahead was missing
  from the key was already stale). Every `HashSet` gets its own `RandomState`
  seed, so the same selection set produced a different key per call - measured 5
  distinct orders out of 6 identical sets in one thread. Each key builds its own
  `DataLoader`, so `todos { user { name } }` over 10 rows ran up to 10 queries
  instead of one batch, intermittently, and correctly only when the lookahead had
  0 or 1 column. Fixed by collecting through a `BTreeSet`: the fix has to order by
  content, not by insertion, because the randomness comes from the `HashMap` and
  `HashSet` sources upstream, which an insertion-ordered set would faithfully
  preserve.
- `gql_load` silently returned near-empty rows outside a relation field. Called
  from a scalar `#[resolver]`, the calling field has no selection set, so the
  lookahead was empty and every column but the join key came back `None`. It now
  errors there, naming `gql_load_with`, the new primitive that takes the columns
  explicitly, built by `gql_look_ahead_cols` or `gql_look_ahead_all`. The signal is
  an empty selection set, not an empty lookahead - gql requires a selection set on
  anything returning an object type, so it cannot fire on a working query, whereas
  an empty lookahead is legitimate for `__typename` or a `#[resolver]` with no
  `sql_dep`. `gql_look_ahead_all` is built from `gql_select`, not `gql_cols`, which
  still holds `#[graphql(skip)]` columns despite what its doc comment used to say.
  The worked example item 22 asked for is in model.md, so that item is closed too.
- `History` stored every column, defeating `#[graphql(skip)]`. `History::add`
  snapshotted the raw row, so enabling `#[model(history)]` on a model holding
  secrets put `password_hashed` and friends into `History.data` as cleartext json,
  reachable through any `#[search(History)]`. Redaction has to happen on write:
  `#[graphql(skip)]` and an authz col policy are both per column, and `data` is one
  opaque column nothing downstream can look inside. `EntityX::history_skip()` is now
  generated per model, `History::add`/`add_many` drop those keys, `data` itself is
  `#[graphql(skip)]`, and `#[history(skip)]` opts a column out of the audit trail
  without hiding it from the api. `History::diff` covers the read side, snapshots
  stay the storage format - a stored delta chain would be silently wrong for good
  the first time a write bypasses history. What is left is the row boundary, now
  item 4.
- Commit was silently swallowed whenever a dataloader ran. `LoaderX` held an
  `Arc<DatabaseTransaction>`, async_graphql keeps the loader alive briefly inside
  the `tokio::spawn`ed batch task after handing rows back, and that task could
  outlive the request and make `Arc::try_unwrap` in `commit` fail. The whole
  request's writes were then dropped while the client saw `data` plus a confusing
  error. `LoaderX` now holds a `Weak` and upgrades only for the duration of a
  batch. Covered by `tests/tx/`.
- A failed `cleanup` left `data` populated, reading as success to any client that
  only checks `data`. The extension now nulls it.
- `#[query(authz(..))]` and `#[query(auth)]` replaced by the generic
  `#[query(check = my_guard)]`, so `macro_proc` no longer hardcodes auth or authz.
- One connection type, `ConnX`, that is either the request transaction or a pooled
  connection, chosen from the operation type before any resolver runs. A query no
  longer opens a transaction it does not need, a subscription never opens one at
  all, and the crud macros inject it as `db` instead of `tx`. Because it is a
  borrow rather than an owning handle, `Arc::try_unwrap` is gone from the commit
  path and with it the whole class of commit failures caused by something holding
  a connection too long, along with the sea-orm drop panic that could follow.
- Subscriptions: `#[subscribe(Model)]`, crud mutations publish after the commit,
  in memory and redis brokers behind `SubscriptionBroker`. The transaction a
  subscription used to leak is closed by `ctx.tx_finish()` after its guards, plus
  a release tied to the stream's own lifetime in `GrandLineExtension::subscribe`.
- Consumer DI boilerplate replaced by `#[model]` attributes: `auth_session`,
  `auth_otp`, `authz_org`, `authz_role`, `authz_user_in_role`, `authz_org_id`.
- `dprint fmt` was silently emptying every `rs` code block in the docs. The exec
  plugin in `dprint.json` was wired to `cargo +nightly fmt --all`, which ignores
  stdin and prints nothing, so the markdown plugin took that empty stdout as the
  formatted block. Reproduced from scratch: four out of four blocks wiped,
  including a plain `fn main()`. Switched to `rustfmt +nightly --edition 2024`,
  which formats stdin properly and leaves fragments it cannot parse alone, and
  restored the four examples that had already been lost in
  `authorization.md` (two of them destroyed by this session's own commit),
  `history.md`, and `schema-collector.md`.
