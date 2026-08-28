# History (audit log)

An opt-in, per-model audit trail: every create/update/(soft or permanent) delete gets appended as a row to a single shared `History` table, storing a JSON snapshot of the row at that point.

## Enabling it

```rs
#[model(history)]
pub struct Todo {
    pub content: String,
}
```

`history` defaults to `false` and is opt-in per model - most models don't need it, turn it on for the ones that matter to audit.

## The `History` row

```rs
#[sql_enum]
pub enum HistoryOperation {
    Create,
    Update,
    Delete,
    PermanentDelete,
}

#[model(updated_at = false, deleted_at = false, by_id = false)]
pub struct History {
    pub entity_type: String, // the owning model's name, e.g. "Todo"
    pub entity_id: String,   // that model's row id
    pub operation: HistoryOperation,
    pub by_id: Option<String>, // who performed it, None if unavailable
    #[graphql(skip)]
    pub data: JsonValue, // row snapshot at that point, see What the snapshot stores
}
```

`History` is a plain model like any other - query it directly:

```rs
History::find()
    .filter(HistoryColumn::EntityType.eq("Todo"))
    .filter(HistoryColumn::EntityId.eq(&id))
    .order_by_asc(HistoryColumn::CreatedAt)
    .all(db)
    .await?;
```

## What the snapshot stores

`data` is the row minus every column the owning model keeps out of the API. Two attributes opt a column out, and both are applied when the snapshot is written, not when it is read:

| Attribute          | Column over GraphQL | Column in `History.data` |
| ------------------ | ------------------- | ------------------------ |
| none               | exposed             | stored                   |
| `#[graphql(skip)]` | hidden              | dropped                  |
| `#[history(skip)]` | exposed             | dropped                  |

```rs
#[model(history)]
pub struct User {
    pub email: String,
    #[graphql(skip)] // hidden from the api, so never snapshotted either
    pub password_hashed: String,
    #[history(skip)] // fine over the api, just not worth retaining
    pub last_seen_ip: String,
}
```

Redaction has to happen on write because nothing downstream can see inside a JSON value: `#[graphql(skip)]` and an authz col policy both work per column, and `data` is one opaque column. That is also why `data` itself is `#[graphql(skip)]` - a `#[search(History)]` would otherwise hand every audited row of every model to any caller that reaches the resolver. Expose it through a resolver of your own, with its own guard, when an audit UI needs it.

`#[history(skip)]` only affects the audit trail. The column is still stored, still queryable, still exposed over GraphQL.

## Diffing two entries

Entries store snapshots rather than deltas, so a diff is computed on read:

```rs
let changes = History::diff(&prev, &next);
// [HistoryChange { col: "content", from: json!("old"), to: json!("new") }, ..]
```

Sorted by column name, top level only - a JSON column counts as changed as a whole. A column dropped from one snapshot but not the other (an attribute added between the two writes) shows up as a change to or from `null`.

Storing deltas instead would be smaller but not safe: any write that bypasses history breaks every reconstruction after it, silently, and produces row states that never existed. A snapshot chain with a hole in it just has a hole.

## When a row gets written

| Write path                                            | Operation recorded             | `by_id`                                                               |
| ----------------------------------------------------- | ------------------------------ | --------------------------------------------------------------------- |
| `am.exec(ctx)` (create/update/soft-delete)            | `Create` / `Update` / `Delete` | `ctx.auth()`, `None` if unauthenticated (requires the `auth` feature) |
| `am.exec_without_ctx(db)` (create/update/soft-delete) | `Create` / `Update` / `Delete` | always `None`                                                         |
| `#[delete(Todo)]` CRUD macro, soft delete (default)   | `Delete`                       | current user if authenticated                                         |
| `#[delete(Todo)]` CRUD macro, `permanent: true`       | `PermanentDelete`              | current user if authenticated                                         |

`exec_without_ctx` only skips the `*_by_id` fields on the model itself (see [Active model helpers](active-model-helpers.md)) - it still writes a `History` row when the model has `history = true`, just with `by_id: None` since there's no `Context` to read the current user from.

History recording and the write it documents share one transaction: `History::add`/`add_many` run inside the same `tx` as the create/update/delete, so a failure anywhere after the history insert rolls the whole transaction back together - there's no window where a history row exists without its corresponding write, or vice versa.

## Known gaps

- **Bulk `Entity::insert_many` bypasses history.** Only the `Vec<AmWrapper<AmCreate, ..>>` exec path (`am_create_many!` + `.exec(ctx)`/`.exec_without_ctx(db)`) records history per row - calling sea-orm's raw `insert_many` directly skips it. See [Design notes](contribution/design-notes.md#known-limitations).
- **No "restore this version" helper.** `History::diff` is the only reader helper - otherwise `History` is a log you query and interpret yourself.
- **`History` has no org column, so a row policy cannot scope it.** Even with `data` skipped, `entity_id`, `by_id` and the timestamps of every audited model are in one table. Don't put a plain `#[search(History)]` behind a guard that isn't as strong as the strongest model you audit.
