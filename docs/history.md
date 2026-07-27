# History (audit log)

An opt-in, per-model audit trail: every create/update/(soft or permanent) delete gets appended as a row to a single shared `History` table, storing a full JSON snapshot of the row at that point.

## Enabling it

```rs
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
    pub entity_type: String,     // the owning model's name, e.g. "Todo"
    pub entity_id: String,       // that model's row id
    pub operation: HistoryOperation,
    pub by_id: Option<String>,   // who performed it, None if unavailable
    pub data: JsonValue,         // full JSON snapshot of the row at that point
}
```

`History` is a plain model like any other - query it directly:

```rs
History::find()
    .filter(HistoryColumn::EntityType.eq("Todo"))
    .filter(HistoryColumn::EntityId.eq(&id))
    .order_by_asc(HistoryColumn::CreatedAt)
    .all(tx)
    .await?;
```

## When a row gets written

| Write path                                            | Operation recorded             | `by_id`                                                               |
| ----------------------------------------------------- | ------------------------------ | --------------------------------------------------------------------- |
| `am.exec(ctx)` (create/update/soft-delete)            | `Create` / `Update` / `Delete` | `ctx.auth()`, `None` if unauthenticated (requires the `auth` feature) |
| `am.exec_without_ctx(tx)` (create/update/soft-delete) | `Create` / `Update` / `Delete` | always `None`                                                         |
| `#[delete(Todo)]` CRUD macro, soft delete (default)   | `Delete`                       | current user if authenticated                                         |
| `#[delete(Todo)]` CRUD macro, `permanent: true`       | `PermanentDelete`              | current user if authenticated                                         |

`exec_without_ctx` only skips the `*_by_id` fields on the model itself (see [Active model helpers](active-model-helpers.md)) - it still writes a `History` row when the model has `history = true`, just with `by_id: None` since there's no `Context` to read the current user from.

History recording and the write it documents share one transaction: `History::add`/`add_many` run inside the same `tx` as the create/update/delete, so a failure anywhere after the history insert rolls the whole transaction back together - there's no window where a history row exists without its corresponding write, or vice versa.

## Known gaps

- **Bulk `Entity::insert_many` bypasses history.** Only the `Vec<AmWrapper<AmCreate, ..>>` exec path (`am_create_many!` + `.exec(ctx)`/`.exec_without_ctx(tx)`) records history per row - calling sea-orm's raw `insert_many` directly skips it. See [Design notes](contribution/design-notes.md#known-limitations).
- **No built-in query helpers beyond the plain model.** There's no "diff between two history rows" or "restore this version" helper - `History` is a log you query and interpret yourself.
