use grand_line::prelude::*;

#[tokio::test]
async fn create_many_inserts_all_episodes_in_one_call() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
            pub season: i64,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode);

    let episodes = am_create_many!(Episode, [
        { title: "Pilot", season: 1 },
        { title: "The Same Old Story", season: 1 },
        { title: "Peter", season: 2 },
    ])
    .exec_without_ctx(&tmp.db)
    .await?;

    let [e0, e1, e2] = episodes.as_slice() else {
        return TestErr::expect("3 episodes should be created");
    };
    pretty_eq!(e0.title, "Pilot", "e0 title should match the input order");
    pretty_eq!(e1.title, "The Same Old Story", "e1 title should match the input order");
    pretty_eq!(e2.season, 2, "e2 season should match the input value");

    let count = Episode::find().count(&tmp.db).await?;
    pretty_eq!(count, 3u64, "create_many should insert all rows in one call");

    tmp.drop().await
}

#[tokio::test]
async fn create_many_with_empty_array_returns_empty_vec() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode);

    let episodes = am_create_many!(Episode, []).exec_without_ctx(&tmp.db).await?;
    pretty_eq!(
        episodes.len(),
        0,
        "create_many with an empty array should return an empty vec",
    );

    tmp.drop().await
}

#[tokio::test]
async fn update_many_applies_distinct_values_per_row() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
            pub season: i64,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode);

    let episodes = am_create_many!(Episode, [
        { title: "Pilot", season: 1 },
        { title: "In Which We Meet Mr. Jones", season: 1 },
    ])
    .exec_without_ctx(&tmp.db)
    .await?;
    let [e0, e1] = episodes.as_slice() else {
        return TestErr::expect("2 episodes should be created");
    };

    let updated = am_update_many!(Episode, [
        { id: e0.id.clone(), season: 2 },
        { id: e1.id.clone(), title: "The Ghost Network" },
    ])
    .exec_without_ctx(&tmp.db)
    .await?;

    let [u0, u1] = updated.as_slice() else {
        return TestErr::expect("2 rows should be updated");
    };
    pretty_eq!(
        u0.title,
        "Pilot",
        "u0 title should stay unchanged when only season is updated",
    );
    pretty_eq!(u0.season, 2, "u0 season should be updated to 2");
    pretty_eq!(
        u1.title,
        "The Ghost Network",
        "u1 title should be updated to The Ghost Network",
    );
    pretty_eq!(
        u1.season,
        1,
        "u1 season should stay unchanged when only title is updated",
    );

    tmp.drop().await
}

#[tokio::test]
async fn soft_delete_many_sets_deleted_at_on_each_row() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode);

    let episodes = am_create_many!(Episode, [
        { title: "Pilot" },
        { title: "The Same Old Story" },
        { title: "The Ghost Network" },
    ])
    .exec_without_ctx(&tmp.db)
    .await?;
    let [e0, e1, e2] = episodes.as_slice() else {
        return TestErr::expect("3 episodes should be created");
    };

    let deleted = am_soft_delete_many!(Episode, [
        { id: e0.id.clone() },
        { id: e1.id.clone() },
    ])
    .exec_without_ctx(&tmp.db)
    .await?;

    let [d0, d1] = deleted.as_slice() else {
        return TestErr::expect("2 rows should be soft deleted");
    };
    pretty_eq!(d0.deleted_at.is_some(), true, "e0 should be soft deleted");
    pretty_eq!(d1.deleted_at.is_some(), true, "e1 should be soft deleted");

    let Some(remaining) = Episode::find_by_id(&e2.id).one(&tmp.db).await? else {
        return TestErr::expect("e2 episode should still exist");
    };

    pretty_eq!(remaining.deleted_at.is_none(), true, "e2 should not be soft deleted");

    tmp.drop().await
}

#[tokio::test]
async fn create_many_with_returning_reflects_db_state() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode);

    let episodes = am_create_many!(Episode, [
        { title: "Pilot" },
        { title: "The Same Old Story" },
    ])
    .returning()
    .exec_without_ctx(&tmp.db)
    .await?;

    let [e0, e1] = episodes.as_slice() else {
        return TestErr::expect("2 episodes should be created");
    };
    pretty_eq!(e0.title, "Pilot", "e0 title should reflect the returning() result");
    pretty_eq!(
        e1.title,
        "The Same Old Story",
        "e1 title should reflect the returning() result",
    );

    tmp.drop().await
}

#[tokio::test]
async fn update_without_id_returns_err() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode);

    let r = am_update!(Episode {
        title: "The Ghost Network"
    })
    .exec_without_ctx(&tmp.db)
    .await;
    pretty_eq!(r.err().is_some(), true, "update without id should return err");

    tmp.drop().await
}

#[tokio::test]
async fn soft_delete_without_id_returns_err() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode);

    let r = am_soft_delete!(Episode).exec_without_ctx(&tmp.db).await;
    pretty_eq!(r.err().is_some(), true, "soft delete without id should return err");

    tmp.drop().await
}

#[tokio::test]
async fn update_many_without_id_on_one_item_returns_err() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode);

    let episodes = am_create_many!(Episode, [{ title: "Pilot" }])
        .exec_without_ctx(&tmp.db)
        .await?;
    let [e0] = episodes.as_slice() else {
        return TestErr::expect("1 episode should be created");
    };

    let r = am_update_many!(Episode, [
        { id: e0.id.clone(), title: "The Same Old Story" },
        { title: "The Ghost Network" },
    ])
    .exec_without_ctx(&tmp.db)
    .await;
    pretty_eq!(
        r.err().is_some(),
        true,
        "update many with one missing id should return err",
    );

    tmp.drop().await
}

#[tokio::test]
async fn soft_delete_many_without_id_on_one_item_returns_err() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode);

    let episodes = am_create_many!(Episode, [
        { title: "Pilot" },
    ])
    .exec_without_ctx(&tmp.db)
    .await?;
    let [e0] = episodes.as_slice() else {
        return TestErr::expect("1 episode should be created");
    };

    let r = am_soft_delete_many!(Episode, [
        { id: e0.id.clone() },
        {},
    ])
    .exec_without_ctx(&tmp.db)
    .await;
    pretty_eq!(
        r.err().is_some(),
        true,
        "soft delete many with one missing id should return err",
    );

    tmp.drop().await
}
