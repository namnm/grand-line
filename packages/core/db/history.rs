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
/// (EntityX::model_name()), entity_id is that model's row id.
#[model(updated_at = false, deleted_at = false, by_id = false)]
pub struct History {
    pub entity_type: String,
    pub entity_id: String,
    pub operation: HistoryOperation,
    pub by_id: Option<String>,
    pub data: JsonValue,
}

impl History {
    /// Build and insert one history log entry for m.
    pub async fn add<E, M, D>(tx: &D, operation: HistoryOperation, model: &M, by_id: Option<String>) -> Res<()>
    where
        E: EntityX<M = M>,
        M: ModelX<E>,
        D: ConnectionTrait,
    {
        #[allow(clippy::use_self)]
        let am = am_create!(History {
            entity_type: E::model_name().to_owned(),
            entity_id: model.get_id(),
            operation,
            by_id,
            data: model.to_json()?,
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
                #[allow(clippy::use_self)]
                let am = am_create!(History {
                    entity_type: E::model_name().to_owned(),
                    entity_id: m.get_id(),
                    operation,
                    by_id: by_id.clone(),
                    data: m.to_json()?,
                })
                .into_am_without_ctx();
                Ok(am)
            })
            .collect::<Res<Vec<_>>>()?;

        Self::insert_many(ams).exec_without_returning(tx).await?;
        Ok(())
    }
}
