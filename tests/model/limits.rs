use grand_line::prelude::*;

// ---------------------------------------------------------------------------
// Offset is bounded, a deep one is a full scan from an ordinary looking query
// ---------------------------------------------------------------------------

#[tokio::test]
async fn offset_is_clamped_to_offset_max() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let c = CoreConfig {
        offset_max: 50,
        ..Default::default()
    };
    let p = Pagination {
        offset: Some(1_000_000),
        limit: None,
    };

    pretty_eq!(p.inner(&c).offset, 50, "a deep offset should be clamped to offset_max");

    tmp_db!(Episode).drop().await
}

#[tokio::test]
async fn offset_below_max_is_left_alone() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let c = CoreConfig::default();
    let p = Pagination {
        offset: Some(7),
        limit: None,
    };

    pretty_eq!(p.inner(&c).offset, 7, "an offset under offset_max should pass through");

    tmp_db!(Episode).drop().await
}

#[tokio::test]
async fn search_resolver_clamps_the_requested_offset() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
        }

        #[search(Episode)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(Episode);
    let c = CoreConfig {
        offset_max: 1,
        ..Default::default()
    };
    let s = schema_q::<EpisodeSearchQuery>(&tmp.db).data(c).finish();

    am_create!(Episode {
        title: "Pilot",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(Episode {
        title: "The Same Old Story",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(Episode {
        title: "The Ghost Network",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test {
        episodeSearch(page: { offset: 1000000 }) {
            title
        }
    }
    ";
    let r = exec_assert_ok(&s, q, None).await;
    let r = r.data.to_json()?;

    pretty_eq!(
        r.arr("/episodeSearch").len(),
        2,
        "offset 1000000 clamped to 1 should skip one of the three records",
    );

    tmp.drop().await
}

// ---------------------------------------------------------------------------
// order_by entry count is bounded
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_order_by_is_capped_to_order_by_max() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let c = CoreConfig {
        order_by_max: 2,
        ..Default::default()
    };
    let order_by = Some(vec![
        EpisodeOrderBy::TitleAsc,
        EpisodeOrderBy::IdAsc,
        EpisodeOrderBy::CreatedAtDesc,
    ]);

    let r = order_by.combine(vec![], &c);

    pretty_eq!(r.len(), 2, "a client order_by list should be capped to order_by_max");
    pretty_eq!(
        r.first().is_some_and(|o| matches!(*o, EpisodeOrderBy::TitleAsc)),
        true,
        "the cap should keep the leading entries, not reorder them",
    );

    tmp_db!(Episode).drop().await
}

#[tokio::test]
async fn app_default_order_by_is_not_capped() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
        }
    }
    use test::*;

    let c = CoreConfig {
        order_by_max: 1,
        ..Default::default()
    };
    let order_by: Option<Vec<EpisodeOrderBy>> = None;
    let app_default = vec![
        EpisodeOrderBy::TitleAsc,
        EpisodeOrderBy::IdAsc,
        EpisodeOrderBy::CreatedAtDesc,
    ];

    pretty_eq!(
        order_by.combine(app_default, &c).len(),
        3,
        "an app own default order_by is deliberate and should not be capped",
    );

    tmp_db!(Episode).drop().await
}
