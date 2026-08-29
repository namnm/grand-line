use super::prelude::*;

/// Kind of history log entry, stored as create/update/delete in the db.
#[sql_enum]
pub enum HistoryOperation {
    Create,
    Update,
    Delete,
    PermanentDelete,
}

/// Shared history log, one row per create/update/delete across every model with
/// #[model(history)] enabled. entity_type is the owning model's name
/// (EntityX::model_name()), entity_id is that model's row id. org_id is the
/// owning model's org_id column, when it has one, the signal #[authz_org_id]
/// requires exactly that name, so a row policy has something to scope the
/// shared audit table with.
#[model(updated_at = false, deleted_at = false, by_id = false)]
pub struct History {
    pub entity_type: String,
    pub entity_id: String,
    pub operation: HistoryOperation,
    pub by_id: Option<String>,
    /// The audited row's org_id, None when the owning model has no org column.
    pub org_id: Option<String>,
    /// Row snapshot, already stripped of every column the owning model keeps out
    /// of the api (see History::snapshot).
    /// Skipped from graphql on purpose: a json blob has no columns, so neither
    /// #[graphql(skip)] nor a col policy can reach inside it. Serve it through an
    /// explicit guarded resolver of your own when an audit ui needs it.
    #[graphql(skip)]
    pub data: JsonValue,
}

/// One column that differs between two snapshots of the same row.
#[derive(Debug, Clone, Serialize)]
pub struct HistoryChange {
    pub col: String,
    pub from: JsonValue,
    pub to: JsonValue,
}

impl History {
    // ------------------------------------------------------------------------
    // Recording
    // ------------------------------------------------------------------------

    /// Build and insert one history log entry for m.
    pub async fn add<E, M, D>(tx: &D, operation: HistoryOperation, model: &M, by_id: Option<String>) -> Res<()>
    where
        E: EntityX<M = M>,
        M: ModelX<E>,
        D: ConnectionTrait,
    {
        let (org_id, data) = Self::snapshot::<E, M>(model)?;
        #[allow(clippy::use_self)]
        let am = am_create!(History {
            entity_type: E::model_name().to_owned(),
            entity_id: model.get_id(),
            operation,
            by_id,
            org_id,
            data,
        })
        .into_am_without_ctx();

        Self::insert(am).exec_without_returning(tx).await?;
        Ok(())
    }

    /// Build and insert many history log entries in one bulk INSERT, same operation
    /// and by_id for every row (e.g. after a bulk create). See add for the single-row version.
    pub async fn add_many<E, M, D>(tx: &D, operation: HistoryOperation, models: &[M], by_id: Option<String>) -> Res<()>
    where
        E: EntityX<M = M>,
        M: ModelX<E>,
        D: ConnectionTrait,
    {
        if models.is_empty() {
            return Ok(());
        }

        let ams = models
            .iter()
            .map(|m| {
                let (org_id, data) = Self::snapshot::<E, M>(m)?;
                #[allow(clippy::use_self)]
                let am = am_create!(History {
                    entity_type: E::model_name().to_owned(),
                    entity_id: m.get_id(),
                    operation,
                    by_id: by_id.clone(),
                    org_id,
                    data,
                })
                .into_am_without_ctx();
                Ok(am)
            })
            .collect::<Res<Vec<_>>>()?;

        Self::insert_many(ams).exec_without_returning(tx).await?;
        Ok(())
    }

    /// The owning row's org_id and its snapshot. The org column is read before
    /// the history_skip stripping: an org scope survives in the dedicated column
    /// even when the model keeps org_id out of the audit json.
    fn snapshot<E, M>(model: &M) -> Res<(Option<String>, JsonValue)>
    where
        E: EntityX<M = M>,
        M: ModelX<E>,
    {
        let mut data = model.to_json()?;
        // #[authz_org_id] requires the column to be named org_id, so a model that
        // declares org scoping is exactly a model with that field
        let org_id = data.get("org_id").and_then(JsonValue::as_str).map(str::to_owned);
        if let Some(o) = data.as_object_mut() {
            for k in E::history_skip() {
                o.remove(*k);
            }
        }
        Ok((org_id, data))
    }

    // ------------------------------------------------------------------------
    // Reading
    // ------------------------------------------------------------------------

    /// Columns that differ between two snapshots of the same row, sorted by column name.
    /// Diffing on read keeps every entry self contained, a stored delta chain would be
    /// wrong for good the first time a write bypasses history.
    /// Top level only, a json column counts as changed as a whole.
    pub fn diff(from: &HistorySql, to: &HistorySql) -> Vec<HistoryChange> {
        let (Some(a), Some(b)) = (from.data.as_object(), to.data.as_object()) else {
            return vec![];
        };

        let mut cols = a
            .keys()
            .chain(b.keys())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        cols.sort();

        cols.into_iter()
            .filter_map(|c| {
                let from = a.get(c).cloned().unwrap_or(JsonValue::Null);
                let to = b.get(c).cloned().unwrap_or(JsonValue::Null);
                if from == to {
                    return None;
                }
                Some(HistoryChange {
                    col: c.clone(),
                    from,
                    to,
                })
            })
            .collect()
    }
}
