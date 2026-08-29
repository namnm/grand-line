# todo

Open issues across the framework. Merged from four audit passes (the original
todo plus todo2/todo3/todo4), renumbered end to end, every item re-verified
against master before being kept. Ordered by priority. Items marked
`[file_upload]` only exist on that branch and block merging it into master.

---

## P0 - authorization fails open, or a merge is blocked

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

**Fix.** Same shape as item 1, the `has_actor` signal: make `authz_row` default to
off when the resolver declares no `check`, rather than keying purely off a global
feature. A resolver that never runs an authz guard has no row policy to apply
anyway. The quick alternative is spelling `authz_row = false` on the four `_file`
resolvers, but that leaves the same trap for the next package.

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

### 7. `gql_update` returns the updated row without re-applying the row policy

**Why.** [`gql_update`](../crates/core/db/entity.rs) runs the write with the
caller's authz row filter, then re-reads the response row with no filter at all:

```rs
let rows_affected = Self::update_many()
    .filter_by_id(id)
    .filter_option(authz_row) // policy applied to the write
    .set(am)
    .exec(tx)
    .await?
    .rows_affected;
...
let r = Self::find().filter_by_id(id).gql_select(ctx)?.one_or_404(tx).await?;
```

The write's `WHERE` is evaluated against the **old** row, so an update that changes
the very column the policy filters on succeeds, and the re-read hands the caller a
row that no longer matches their policy. With `org_id = 'orgA'`, a user sets
`org_id` to `'orgB'` and gets back the row now sitting in orgB. The soft-delete
branch of `gql_delete` has the same shape for its post-delete history read, smaller
exposure since the row is leaving the active domain.

**Fix.** Apply the same filter to the post-write read, and treat a miss as the
authorization error rather than a 404:

```rs
let r = Self::find()
    .filter_by_id(id)
    .filter_option(authz_row)
    .gql_select(ctx)?
    .one(tx)
    .await?
    .ok_or_else(|| authz_err.clone())?;
```

Moving a row out of your own scope arguably should not succeed at all, so also
consider rejecting the write when the _new_ value would fall outside the policy.

---

## P1 - correctness bugs, latent but severe when hit

### 9. A row policy that references `deleted_at` is silently cancelled

**Why.** `Filter::include_deleted` decides from three inputs, and the authz row
filter is not one of them:

```rs
self.include_deleted || include_deleted.unwrap_or_default() || filter.is_some_and(|f| f.has_deleted_at())
```

`filter` there is the **client** filter. The authz row filter reaches the query
through `Search::add_option`, which calls `Filter::add_option`, which only ANDs
`c.into_condition()` and never touches `include_deleted` - even though
`impl From<F> for Filter` does set it from `has_deleted_at()`. So an admin row
policy meant to include soft-deleted rows gets `deleted_at IS NULL` added on top
and returns nothing, with no signal to the policy author. `gql_search`,
`gql_count`, `gql_detail` and `gql_load_with` all share the shape.

**Fix.** Consult the authz row filter in the same decision. Keep it as a separate
`Option<Self::F>` alongside `extra` rather than folding it into the condition
early, then:

```rs
let inc = extra.include_deleted(include_deleted, filter.as_ref())
    || authz_row.as_ref().is_some_and(|f| f.has_deleted_at());
```

Until then, document that a row policy must not reference `deletedAt`.

### 10. Update and soft-delete still operate on soft-deleted rows

**Why.** `Self::find()` does not exclude soft-deleted rows on its own, that is what
`.include_deleted(false)` is for, and the mutation path never chains it:
`gql_mutation_check_id` looks the id up with a bare `Self::find().filter_by_id(id)`,
`gql_update` writes with `update_many().filter_by_id(id).filter_option(authz_row)`,
and the soft-delete branch of `gql_delete` has no `deleted_at IS NULL` guard
either. `include_deleted` is exposed on search/count/detail but has no equivalent
on update/delete.

So a row invisible to every normal query is still updatable by id, soft-deleting
twice rewrites `deleted_at` and its audit fields again, and history and
subscriptions both emit a second "delete" for an entity that was already deleted.

**Fix.** Default update and soft-delete to `deleted_at IS NULL`. Permanent delete
should keep seeing deleted rows deliberately. If a recovery workflow is wanted,
give it an explicit `restore` mutation rather than letting a plain update resurrect
a deleted row by accident.

### 11. A subscription keeps the permissions it opened with

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
delivery, and a detached job that started before a change landed and should notice
before it writes. Design it as a general resource version, keyed by principal and
by resource, not as an authz specific hack.

### 12. A commit that succeeded is reported to the client as a failed mutation

**Why.** `GrandLineExtension::execute` commits, then publishes the queued
subscription events, and pushes any publish error onto the response:

```
mutation -> SQL ok -> COMMIT ok -> publish fails -> client sees a GraphQL error
```

A client is entirely reasonable to retry what looks like a failed mutation, and a
non-idempotent one then runs twice over a first attempt that did persist. With
several queued events, some may have published before a later one failed.

**Fix.** Decide what subscription delivery is. If it is best effort, a publish
failure after the commit should log and increment a metric, not turn a successful
mutation into a failure. If it must be reliable, use a transactional outbox: write
the event rows inside the same transaction and let a relay publish and retry
independently. For a backend framework the outbox is the cleaner answer, and it
also removes the "commit then publish" window entirely.

### 13. `AuthOtpImpl`'s signature lets a consumer lose the attempt increment

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

### 14. `auth_otp_ensure_re_request` deletes on db and re-creates on tx

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

### 15. `ilike` is emitted from a compile-time feature and breaks on sqlite and mysql

**Why.** [`filter.rs`](../crates/macro_proc/model/filter.rs) picks the string
operators with `cfg!(feature = "postgres")` on the **proc-macro** crate:

```rs
if cfg!(not(feature = "postgres")) {
    push(f, struk, query, "like")?;
    push(f, struk, query, "not_like")?;
}
if cfg!(feature = "postgres") {
    push(f, struk, query, "ilike")?;
    push(f, struk, query, "not_ilike")?;
}
```

Same class as item 1: that flag reflects cargo feature unification, not the
database the process connects to. The workspace default enables postgres, so a
build that later points at sqlite or mysql still exposes `contentILike`, which
emits `ILIKE` and is a syntax error on both. `independently.sh` hides it by running
each suite with `--no-default-features` plus exactly one backend.
[filtering-sorting.md](filtering-sorting.md) documents the operator list without
mentioning the feature dependency.

**Fix.** Emit `like`/`not_like` unconditionally, they are valid everywhere,
postgres included. For `ilike`, either document it as postgres-only and have the
condition builder return a clear error on another backend, or drop it and let apps
use `like` with a lowercased column. Either way, stop reading `cfg!` on the
proc-macro crate for a per-connection decision.

### 16. `insert_many_with_returning` branches on `cfg!(feature = "postgres")`

**Why.** [`am_create_many.rs`](../crates/core/db/am_create_many.rs) picks the bulk
create strategy from the compiled feature rather than the connection. With
`returning()` opted in, a build where feature unification pulled postgres in but
whose process runs against mysql calls `exec_with_returning` on a database with no
`RETURNING`, and on sqlite the outcome depends on the sea-orm and sqlx versions in
the lockfile. The function already receives `tx: &D where D: ConnectionTrait`.

**Fix.** Branch on the connection, which is what `get_database_backend()` is for:

```rs
if tx.get_database_backend() == DbBackend::Postgres {
    let models = E::insert_many(ams).exec_with_returning(tx).await?;
    return Ok(models);
}
```

The fallback path is correct on every backend, so this also makes a
`--features postgres` build safe to point anywhere.

### 17. A `#[resolver]` returning an object type still under-selects silently

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

## P2 - robustness and missing patterns

### 18. `AuthOtpContext` increments before verifying the secret

**Why.** [`otp_context.rs`](../crates/auth/context/otp_context.rs) calls
`increment` before checking the code or the secret. The otp row id is handed to
the client by `register`, so anyone holding an id can burn a legitimate user's
attempts and lock them out.

**Fix.** Verify the secret first and only then consume an attempt. Keep the single
opaque `OtpResolveInvalid` for every failure path so this does not become an
oracle for which part was wrong.

### 19. A `*` col policy entry beats a stricter per-operation entry

**Why.** [`authz_without_cache`](../crates/authz/context/cache_context.rs) reads
the wildcard first:

```rs
m.col_policy.get("*").or_else(|| m.col_policy.get(self.field_impl().name()))
```

The comment says it is intentional, because `col_policy` is allow-only and there is
no deny entry that could express "everything via `*` except this one operation".
That is a coherent reason, but the resulting shape reads backwards to anyone
writing a policy: `*` granting broadly plus `delete` granting narrowly looks like a
specialization, and is silently the opposite.

**Fix.** Either flip the precedence so an exact operation entry wins over the
wildcard fallback, which is what every allow-list system people have used before
does, or keep the precedence and reject a config carrying both `*` and a specific
entry for the same field at startup. A validator warning is the minimum, since the
current behavior is invisible until an audit.

### 22. The redis publish connection is cached forever, with no reconnect

**Why.** `RedisBroker` in
[`broker_redis.rs`](../crates/core/subscription/broker_redis.rs) holds the publish
connection in a `OnceCell<MultiplexedConnection>`, and `get_or_try_init` returns the
stored value forever. If redis restarts or the connection drops, every later
publish fails against the stale handle and never re-establishes, so one blip
becomes a permanent subscription outage on every instance until the process is
restarted.

**Fix.** On a publish error, drop the cached connection and retry once with a fresh
one, or use a connection type with built-in reconnect and verify it actually
reconnects. Add a metric for publish failures so the outage is visible.

### 23. A redis subscribe failure silently ends the stream

**Why.** The subscribe side of the same broker wraps each setup step in `.ok()?`:

```rs
let client = Client::open(url).ok()?;
let mut pubsub = client.get_async_pubsub().await.ok()?;
pubsub.subscribe(channel).await.ok()?;
```

so a connection failure ends the stream instead of surfacing an actionable error,
and a payload that fails to deserialize is dropped just as quietly. A transient
redis outage makes live subscriptions disappear with no explanation on either
side, and there is no retry or reconnect.

**Fix.** Make the broker abstraction carry errors, `Stream<Result<SubscriptionEvent>>`
in shape, and give the redis implementation reconnect with bounded exponential
backoff plus metrics for connect and decode failures.

### 24. `authz_with_cache` holds the cache mutex across the whole role lookup

**Why.** In [`cache_context.rs`](../crates/authz/context/cache_context.rs) the
`m.lock().await` guard is still alive when `authz_without_cache(check).await` runs,
and that call does a header read, an `auth()` session resolution, a db query
through `find_matching`, and a col policy check. Every other authz-guarded root
resolver in the same request queues on that mutex for the whole duration, so two
sibling aliased root fields on a read run their guards serially - exactly the
concurrency the pooled-connection design advertises. Not a deadlock, the inner
calls never re-lock this map.

**Fix.** Use the shape `CacheContext::cache` already established: take the map lock
only to fetch or insert a per-key cell, drop it, then `get_or_try_init` the cell
around the lookup. Concurrent first callers for the same field then share one
lookup instead of queueing.

### 25. `has_many` and `many_to_many` are N+1 in the number of parents

**Why.** `has_one` / `belongs_to` go through `gql_load` and its dataloader.
`has_many` and `many_to_many` call `gql_search` once per parent
([relation.rs](../crates/macro_proc/model/relation.rs)), so listing 100 users and
their posts is ~101 queries. For a GraphQL framework this is the capability gap
people notice first.

**Fix.** A batch loader keyed by parent id, `WHERE parent_id IN (..)` then partition
by parent, and the join-table equivalent for `many_to_many`. The hard part is that
the loader key must carry filter, order_by, include_deleted and authz row, and that
per-parent pagination needs window functions. A heuristic that batches the simple
shape and falls back to per-parent when pagination or a custom resolver is involved
captures most of the benefit.

### 26. Dropping a transaction while its connection is locked panics

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

### 27. Opening the transaction holds its mutex across `db.begin()`

**Why.** `GrandLineData::tx_begin` holds the `tx` mutex for the whole
`db.begin().await`. When the pool is exhausted every sibling resolver asking for a
connection on a mutation queues behind it, and the symptom from outside is an
unexplained hanging request. Queries are unaffected, they never call it.

**Fix.** Not incorrect, so low priority, but worth either releasing the lock around
`begin` (accepting that two racing callers may open one transaction too many and
discarding the loser) or documenting the behavior so the queue is recognizable when
diagnosing.

### 28. No timeout bounds the request transaction

**Why.** `cleanup` only runs once `next.run()` returns, so a resolver making an
external call with no timeout holds the transaction and its connection open
indefinitely. This is inherent to one transaction per request, not a bug, but
nothing in the framework or the docs bounds it.

**Fix.** Document the recommendation to set `statement_timeout` and
`idle_in_transaction_session_timeout` on the database side.

### 29. A password reset leaves every existing session valid (saas example)

**Why.** [`forgot_resolve.rs`](../examples/saas/src/auth/forgot_resolve.rs) stores
the new password hash and creates a fresh login session, but does nothing to the
user's other `LoginSession` rows. Anyone holding a bearer token or cookie from
before the reset keeps full access afterwards, which defeats most of the point of
the flow. The framework cannot fix this generically: the session contract
(`AuthSessionImpl::find` by id and secret) has no generation or epoch column that
bulk revocation could key off, and authentication.md never says invalidating
sessions on a credential change is the app's job.

**Fix.** In the example, delete the user's other sessions inside `forgot_resolve`
alongside the password update. In the docs, make session invalidation on a
credential change an explicit app duty. Consider a session generation column on the
`#[auth_session]` contract as a roadmap item so the default impl can enforce it.

### 31. `[file_upload]` hardening for the file package

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

## P3 - footguns, polish and documentation

### 33. `gen_auth_by_id` turns every auth error into "no actor"

**Why.** [`auth.rs`](../crates/macro_proc/utils/auth.rs) emits
`ctx.auth().await.ok()`, so a database error or a malformed session becomes `None`
and the history entry records no actor. The audit trail silently loses the "who"
for writes that did have an authenticated user, with nothing logged to say why.
Only the genuine "unauthenticated" case should produce `None`.

**Fix.** Match on the error and return `None` only for the not-authenticated code,
logging the rest. At minimum log at warn when `ctx.auth()` fails for any other
reason, so a broken auth lookup does not masquerade as anonymous writes.

### 35. i18n calls itself ICU MessageFormat but is a subset, and fails silently

Three related gaps in [`intl.rs`](../crates/i18n/libs/intl.rs), all the same shape:
input that looks like valid ICU produces partially rendered output instead of an
error.

- **Plural bodies are not re-rendered.** After picking a case the code does
  `out.push_str(&t.replace('#', &count.to_string()))` and pushes the result
  directly, so `{count, plural, one{{name} has # item} other{..}}` picks the right
  branch and leaves `{name}` as literal text.
- **An unknown formatter type falls back to raw.** `parse_placeholder` maps
  anything unrecognized to `Ph::Raw`, so `{amount, numbr}` prints the unformatted
  value with no signal that the type was misspelled.
- **No apostrophe escaping.** ICU uses a single quote to escape braces, `'{name}'`
  meaning literal `{name}`. The parser does not handle quotes at all, so prose
  containing an apostrophe before a brace is parsed as a placeholder.

**Fix.** Pick a contract and enforce it. Either implement the recursive render,
the type validation and the quoting rule properly, or rename the contract to
"ICU-like subset" in the docs and **reject** the syntax that is not supported
rather than half-rendering it. Erroring on an unknown formatter type is the
cheapest first step and catches the most likely mistake.

### 36. `grand_line_build` silently skips files it cannot read or parse

**Why.** `scan_dir` swallows three failure classes: `fs::read_dir` errors,
`fs::read_to_string` errors (`unwrap_or_default`), and `parse_file` errors, each
producing zero resolvers with no warning. The `resolve_dirs` check added in this
session catches a missing directory, but a directory that exists yet is unreadable,
or a file that fails to parse, still yields a schema quietly missing resolvers with
a green build. A resolver dropped this way becomes a runtime "unknown field" that
could have been a build error.

**Fix.** Emit `cargo:warning=` naming the path for a `read_dir` or `read_to_string`
failure. A `parse_file` failure is expected for a file that will not compile
anyway, so a warning is enough there too, but the read failures should be loud.

### 37. `org_invitation_reject` contradicts its own comment (saas example)

**Why.** [`invitation_resolve.rs`](../examples/saas/src/authz/invitation_resolve.rs)
marks the resolver `#[mutation(check = authenticated)]` while its body comment says
"No authentication required (mirrors a plain unsubscribe-style link). Proof of
ownership is the id+secret+otp challenge itself, not a session." Both cannot be
right. As written the guard is redundant for security, `auth_otp_ensure_resolve`
already gates on id+secret+otp, but it does reject the unauthenticated flow the
comment describes.

**Fix.** Decide which behavior the example models, then fix the other half, and pin
the choice with a saas test.

### 38. `many_to_many` join-table `include_deleted` is inconsistent

**Why.** In [`relation_shape.rs`](../crates/macro_proc/model/relation_shape.rs),
`many_to_many_condition` passes the relation's `include_deleted` through to the
join-table subquery, while `many_to_many_filter` (the `_some` / `_none` / `_every`
fields) hardcodes `include_deleted(false)` for it. A soft-deleted join row
therefore counts when fetching the list with `includeDeleted: true` but never
counts for the relation filters, so `posts_some` and the `posts` list can disagree
about the same rows with no signal.

**Fix.** Align them. If soft-deleted join rows should always be invisible, drop the
passthrough in `many_to_many_condition` too. If the difference is deliberate, name
it in a comment and in [relationships.md](relationships.md).

### 39. `DefaultOtpImpl::reset` misses `include_deleted(false)`

**Why.** `find` and `increment` in [`otp.rs`](../crates/auth/models/otp.rs) both
exclude soft-deleted rows, `reset` does not. For a consumer OTP model that keeps
`deleted_at` (the example disables it, the macro only requires the listed columns),
a resolve on a soft-deleted row would still zero its attempt counter.

**Fix.** Add `.include_deleted(false)` to the reset query, matching its siblings.

### 40. Small docs and message fixes

- `OtpReRequestTooSoon` renders as "otp is not yet to re-request", which reaches
  clients as-is since the variant is `#[client]`. Something like "otp re-request is
  still in cooldown, try again later".
- [error-handling.md](error-handling.md) links `examples/saas/src/err.rs`, the file
  is [`examples/saas/src/utils/err.rs`](../examples/saas/src/utils/err.rs).
- [debug-macros.md](debug-macros.md) says `debug_macro_cli` prints "with syntax
  highlighting via prettyplease". prettyplease only formats; the single
  `bright_black` from the colored crate is applied to the whole output. Reword to
  "pretty-printed with prettyplease".
- [design-notes.md](contribution/design-notes.md) still lists, under Known
  limitations, "No subscriptions yet. EmptySubscription is used throughout", and
  roadmap item 6 calls subscriptions "currently the largest capability gap".
  Subscriptions shipped. Roadmap item 8 also lists `LICENSE` as missing, the file
  exists, and CONTRIBUTING effectively exists as docs/contribution.md. CI and a
  changelog genuinely are still missing, keep those.
- CLAUDE.md's Formatting Rules table reads "Semicolon: colon (,)", which names a
  colon while showing a comma. Should be "comma (,)".

### 41. The saas example's `main.rs` wiring has no test

**Why.** Every saas test builds the schema itself and attaches the `HeaderMap`
by hand, so `graphql_handler` and the router in `main.rs` are compiled but never
executed. That is how the item 30 change silently dropped `.data(headers)` from
the handler: it compiled, the whole suite stayed green, and every `ctx` header
helper (bearer token, cookie, ip, ua) would have failed at runtime with
`CtxHeaders404` masked as an internal server error. Caught in review, not by a
test.

**Fix.** Export the router builder from the example's lib (`pub fn app(schema)`)
so `main` is a thin `serve(app(..))`, then drive it in a test the way
[`tests/auth/http_layer.rs`](../tests/auth/http_layer.rs) already drives a router:
one request through the real router asserting an authenticated call works. That
covers the header plumbing, the layer, and their ordering in one go.

### 42. Close the `From<async_graphql::Error> for GrandLineErr` request

**Why not.** The case that prompted it was reaching for the raw db handle, and
`ctx.db().await?` already covers that. For the general case,
`ctx.data_opt_impl::<T>()` returns an `Option`, so `.ok_or(MyErr)?` works with no
conversion. A blanket `From` would be actively harmful: `async_graphql::Error` is
an opaque client-facing type, and auto-converting it invites internal messages to
leak past the framework's deliberately opt-in `#[client]` error model.

**Fix.** Document `ctx.db()` and `data_opt_impl` in [resolvers.md](resolvers.md)
instead, and drop the request.

---

## Verified as already fixed, dropped from the merge

- **The `nongdan-dev` repo urls** flagged in todo4 are gone from README.md and
  docs, only the audit file itself still mentioned them.
- **`get_ip`'s undocumented `x-socket-addr` requirement** is documented in
  authentication.md now. What remains is the axum layer, kept as item 30.

---

## Fixed in this session

Recorded so they are not re-reported.

- **A typo in a row policy filter is an authorization bypass (todo item 4).**
  A filter arriving from the authorization boundary is validated strictly:
  `crates/core/db/filter.rs` gains `FilterKeys`, the model macro implements it
  for every generated filter with the serde field names, and
  `authz_row_get_filter` rejects unknown keys (`RowPolicyFilterKey`) and empty
  objects (`RowPolicyFilterEmpty`) before deserializing. A typo in policy data
  can no longer become an empty filter, i.e. an empty WHERE, i.e. every row.
  The key check recurses through `and`/`or`/`not`, whose branches hold the same
  filter type: validating only the top level left the identical typo silently
  dropped one level down, and a branch that deserializes to an empty filter
  matches every row, so the whole `OR` does too.
- **A row policy whose handler returns `None` fails open (todo item 5).**
  `AuthzConfig` gains `allow_unhandled_row_policy: bool`, default `false`. A
  policy entry whose handler declines now errors with `RowPolicyUnhandled`;
  no policy entry still means no filter. The old lenient behavior is an
  explicit opt-in.
- **A second authz guard on the same resolver is silently skipped (todo
  item 6).** The per-request authz cache stores one entry per `(check, result)`
  under the field's alias, and `authz_with_cache` only serves a hit whose
  `AuthzEnsure` equals the current check. Two guards with different
  requirements each run; two equal checks still share one lookup.
- **`History` has no org column (todo item 8).** `History` gains
  `org_id: Option<String>`, populated from the owning model's `org_id` column
  (the name `#[authz_org_id]` requires) before the snapshot is stripped of
  history-skipped columns. A row policy can now scope the shared audit table.
- **The write/transaction decision reads the whole document (todo item 20).**
  `prepare_request` records the request's `operationName` on `GrandLineData`,
  and `parse_query` classifies only the selected operation. A named query next
  to an unused mutation no longer opens and pins a transaction.
- **`tx_finish` commits but never spawns detached jobs (todo item 21).**
  `tx_finish` now calls `detached_spawn` after a successful commit, the same
  rule as `cleanup`. A subscription resolver's `ctx.detach` job runs instead
  of leaking in the queue forever.
- **`x-socket-addr` has no framework-provided source (todo item 30).**
  `_http_axum` ships `socket_addr_layer`, an axum middleware that fills
  `x-socket-addr` from `ConnectInfo<SocketAddr>` (overwriting anything a
  client sent) and passes requests without connect info through untouched.
  The saas example uses it instead of hand-rolling the header. Its handler still
  has to put the `HeaderMap` into the request data, the layer only fills a
  header and cannot inject the map that `HttpAxumContext::get_headers` reads.
- **Generated resolvers lose their doc comments from the schema (todo item
  32).** `ResolverTyItem` carries the annotated fn's `#[doc]` attributes
  through to `ResolverTy::docs()`, so `///` comments on `#[query]`,
  `#[mutation]` and crud resolvers become GraphQL field descriptions. Pinned
  by an SDL test.
- **The session cookie hardcodes `Secure` with no opt-out (todo item 34).**
  `HttpConfig` gains `cookie_secure: bool`, default `true`, passed through in
  `set_cookie` like `same_site` and `path`.

- **`data` is now nulled when a transaction was actually rolled back** (the open
  decision). `cleanup` returns whether it rolled anything back, and the extension
  nulls `data` only then. A query keeps graphql's partial success untouched: it
  opens no transaction, so an error in one field undid nothing. Documented in
  resolvers.md.
- **The session cookie set no `SameSite` and no `Path`.** Both now come
  from `HttpConfig`, with `SameSite=Lax` and `Path=/` as explicit defaults, applied
  to every cookie `set_cookie` writes rather than only the session one. They live
  on `HttpConfig` rather than `AuthConfig` as the item suggested, because
  `set_cookie` is in `_http` and cannot see `_auth`, and the attributes are a
  property of the deployment, not of auth.
- **No supported way to keep working after the mutation returns.**
  `ctx.detach()` queues a job on `GrandLineData`, spawned only after a successful
  commit and dropped on a rollback. The closure is handed an
  `Arc<DatabaseConnection>`, so it cannot capture the request transaction.
- **`grand_line_build` reported errors cargo ignored.** Now uses
  `println!("cargo::error=..")` (two colons, read from stdout, fails the build)
  plus `process::exit(1)`, rather than the `panic!` the item suggested: it fails
  the build just as hard, without a backtrace, and keeps the no-panic rule. Scan
  dirs are validated with `resolve_dirs`, which names the stale path. This also
  surfaced that the crate's doc examples were only passing because `generate()`
  silently did nothing, they are `no_run` now.
- **Unbounded `offset` and `order_by`.** `CoreConfig` gained `offset_max`
  (10_000) and `order_by_max` (5). Only the client supplied `order_by` is capped,
  an app's own default is deliberate.
- **`get_ip` trusted client supplied headers.** Replaced the fallback
  chain with `HttpConfig::ip_source`: `SocketAddr` by default, or
  `Proxy { header, hops }` counting from the right of a comma separated list.
  `init_common_headers` in test_utils now sets `x-socket-addr`, which is what a
  real handler provides.
- **A missing `User-Agent` blocked login.** `get_ua` returns what it
  found, `HeaderUa404` is gone.
- **`arithmetic_side_effects` allowed workspace wide.** Global allow
  dropped, clippy is clean with it on. Three sites turned out to be real: the
  `Option<..>` strip in `unwrap_option_str` (now `strip_prefix`/`strip_suffix`,
  which also checks the closing bracket), the otp `remaining_attempt` subtraction
  (now saturating), and a test resolver adding two client supplied `i64`. The rest
  are narrow `#[allow]`s with reasons, mostly timestamp shifts by app config and
  byte indices in the i18n parser.
- **Documentation gaps.** resolvers.md now has the `ctx.db_pool()`
  rollback and sqlite locking caveats, the detached job section, and what a
  rollback does to the response. model.md has the worked example of a computed
  field reading a related row with batching.
- **`GQL_SELECT` listed `#[graphql(skip)]` columns**, found by a test written for
  `gql_look_ahead_all`. A skipped field has no resolver so no client can name it,
  making the entry unreachable, but it meant the map was a list of every column
  rather than the reachable ones. `GQL_COLS` still holds them, an `sql_dep` on a
  skipped column still resolves.
- **The dataloader cache key was unstable, so batching silently fell back to n+1.**
  Found while working the gql_load item (now item 17), and the more severe half of it. `gql_look_ahead_of`
  collected through a `HashSet` and iterated it, and `ColumnX::to_loader_key`
  writes that order into the key (the claim that the lookahead was missing from
  the key was already stale). Every `HashSet` gets its own `RandomState`
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
  The worked example the docs item asked for is in model.md, closed with it.
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
  item 8.
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
