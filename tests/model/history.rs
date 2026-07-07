use grand_line::prelude::*;

// ---------------------------------------------------------------------------
// History recorded for each mutating operation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn history_record_create() -> Res<()> {
    mod test {
        use super::*;

        #[model(history)]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode, History);

    let e = am_create!(Episode {
        title: "Pilot",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    // exec_without_ctx has no ctx/actor, but still auto-records history
    let Some(h) = History::find().one(&tmp.db).await? else {
        return TestErr::expect("auto-recorded history entry should exist");
    };

    pretty_eq!(h.entity_type, "Episode", "history entity_type should be the model name");
    pretty_eq!(
        h.entity_id,
        e.id,
        "history entity_id should match the created episode id",
    );
    pretty_eq!(
        h.operation,
        HistoryOperation::Create,
        "history operation should be Create",
    );
    pretty_eq!(
        h.by_id.is_none(),
        true,
        "exec_without_ctx has no actor, by_id should be none",
    );
    pretty_eq!(
        h.data.str("/title"),
        "Pilot",
        "history data snapshot should capture the title",
    );

    tmp.drop().await
}

#[tokio::test]
async fn history_record_create_many() -> Res<()> {
    mod test {
        use super::*;

        #[model(history)]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode, History);

    let episodes = am_create_many!(Episode, [
        { title: "Pilot" },
        { title: "The Same Old Story" },
    ])
    .exec_without_ctx(&tmp.db)
    .await?;
    let [e0, e1] = episodes.as_slice() else {
        return TestErr::expect("2 episodes should be created");
    };

    // Bulk exec_without_ctx skips history entirely, unlike single-row
    // exec_without_ctx which still auto-records with by_id none (see
    // history_record_create above). This is a known, tracked gap, not a bug,
    let exists = History::find().exists(&tmp.db).await?;
    pretty_eq!(
        exists,
        false,
        "exec_without_ctx should not auto-record history for create_many",
    );

    let by_id = Some("olivia_dunham".to_owned());
    History::add_many(&tmp.db, HistoryOperation::Create, &episodes, by_id.clone()).await?;

    let history = History::find().all(&tmp.db).await?;
    pretty_eq!(history.len(), 2, "history should have one entry per created episode");

    let Some(h0) = history.iter().find(|h| h.entity_id == e0.id) else {
        return TestErr::expect("history entry for e0 should exist");
    };

    pretty_eq!(h0.entity_type, "Episode", "h0 entity_type should be the model name");
    pretty_eq!(h0.operation, HistoryOperation::Create, "h0 operation should be Create");
    pretty_eq!(h0.by_id, by_id, "h0 by_id should match the batch actor");
    pretty_eq!(
        h0.data.str("/title"),
        "Pilot",
        "h0 data snapshot should capture the title",
    );

    let Some(h1) = history.iter().find(|h| h.entity_id == e1.id) else {
        return TestErr::expect("history entry for e1 should exist");
    };

    pretty_eq!(
        h1.data.str("/title"),
        "The Same Old Story",
        "h1 data snapshot should capture the title",
    );

    tmp.drop().await
}

#[tokio::test]
async fn history_record_update() -> Res<()> {
    mod test {
        use super::*;

        #[model(history)]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode, History);

    let e = am_create!(Episode {
        title: "Pilot",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let e = am_update!(Episode {
        id: e.id,
        title: "The Same Old Story"
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    // create and update both auto-record, pick the update entry
    let history = History::find().all(&tmp.db).await?;
    pretty_eq!(history.len(), 2, "create and update should each auto-record one entry");

    let Some(h) = history.iter().find(|h| h.operation == HistoryOperation::Update) else {
        return TestErr::expect("update history entry should exist");
    };

    pretty_eq!(
        h.entity_id,
        e.id,
        "update history entity_id should match the updated episode id",
    );
    pretty_eq!(
        h.by_id.is_none(),
        true,
        "exec_without_ctx has no actor, by_id should be none",
    );
    pretty_eq!(
        h.data.str("/title"),
        "The Same Old Story",
        "history data snapshot should capture the new title",
    );

    tmp.drop().await
}

#[tokio::test]
async fn history_record_soft_delete() -> Res<()> {
    mod test {
        use super::*;

        #[model(history)]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode, History);

    let e = am_create!(Episode {
        title: "The Arrival",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    Episode::soft_delete_by_id(&e.id)?.exec(&tmp.db).await?;

    // query after soft delete to get the deleted_at snapshot
    let Some(deleted) = Episode::find_by_id(&e.id).one(&tmp.db).await? else {
        return TestErr::expect("episode should still exist after soft delete");
    };

    pretty_eq!(deleted.deleted_at.is_some(), true, "episode should be soft deleted");

    // soft_delete_by_id is a raw bulk update (EntityX::soft_delete_by_id), it doesn't go
    // through AmSoftDelete's exec_without_ctx, so it needs an explicit history entry
    History::add(&tmp.db, HistoryOperation::Delete, &deleted, None).await?;

    // create (auto-recorded) + delete (recorded above), pick the delete entry
    let history = History::find().all(&tmp.db).await?;
    pretty_eq!(
        history.len(),
        2,
        "create (auto-recorded) and delete should each record one entry",
    );

    let Some(h) = history.iter().find(|h| h.operation == HistoryOperation::Delete) else {
        return TestErr::expect("soft delete history entry should exist");
    };

    pretty_eq!(
        h.entity_id,
        e.id,
        "soft delete history entity_id should match the deleted episode id",
    );
    pretty_eq!(
        h.data.ptr("/deleted_at").is_null(),
        false,
        "history snapshot should capture deleted_at",
    );

    tmp.drop().await
}

#[tokio::test]
async fn history_record_delete_permanent() -> Res<()> {
    mod test {
        use super::*;

        #[model(history)]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode, History);

    let e = am_create!(Episode {
        title: "In Which We Meet Mr. Jones",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let by_id = Some("peter_bishop".to_owned());
    History::add(&tmp.db, HistoryOperation::PermanentDelete, &e, by_id.clone()).await?;

    Episode::delete_many().filter_by_id(&e.id).exec(&tmp.db).await?;

    // gone from main table
    pretty_eq!(
        Episode::find_by_id(&e.id).exists(&tmp.db).await?,
        false,
        "episode should be permanently deleted from the main table",
    );

    // preserved in history: create (auto-recorded) + delete (recorded above)
    let history = History::find().all(&tmp.db).await?;
    pretty_eq!(history.len(), 2, "history should survive the permanent delete");

    let Some(h) = history
        .iter()
        .find(|h| h.operation == HistoryOperation::PermanentDelete)
    else {
        return TestErr::expect("permanent delete history entry should exist");
    };

    pretty_eq!(
        h.entity_id,
        e.id,
        "permanent delete history entity_id should match the deleted episode id",
    );
    pretty_eq!(
        h.by_id,
        by_id,
        "permanent delete history by_id should match the given actor",
    );
    pretty_eq!(
        h.data.str("/title"),
        "In Which We Meet Mr. Jones",
        "history data snapshot should capture the title",
    );

    tmp.drop().await
}

// ---------------------------------------------------------------------------
// History disabled by default
// ---------------------------------------------------------------------------

#[tokio::test]
async fn history_flag_false_by_default() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Observer {
            pub name: String,
        }
    }
    use test::*;

    pretty_eq!(
        Observer::has_history(),
        false,
        "model without history flag should not have history",
    );

    tmp_db!(Observer).drop().await
}
