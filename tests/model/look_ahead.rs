use grand_line::prelude::*;

// ---------------------------------------------------------------------------
// Stable order, the data loader cache key is built from it
// ---------------------------------------------------------------------------

#[tokio::test]
async fn look_ahead_all_order_is_stable() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
            pub season: i32,
            pub network: String,
            pub observer_seen: bool,
        }
    }
    use test::*;

    let first = EpisodeColumn::Id.to_loader_key(&Episode::gql_look_ahead_all(), "");

    // every source behind the look ahead is a HashMap or HashSet, whose iteration
    // order differs per instance, so one comparison could pass by luck
    for _ in 0..20 {
        let key = EpisodeColumn::Id.to_loader_key(&Episode::gql_look_ahead_all(), "");
        pretty_eq!(key, first, "loader key should be the same on every call");
    }

    tmp_db!(Episode).drop().await
}

#[tokio::test]
async fn look_ahead_cols_order_follows_content_not_argument() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
            pub season: i32,
        }
    }
    use test::*;

    let asc = Episode::gql_look_ahead_cols(&[EpisodeColumn::Season, EpisodeColumn::Title]);
    let desc = Episode::gql_look_ahead_cols(&[EpisodeColumn::Title, EpisodeColumn::Season]);

    pretty_eq!(
        EpisodeColumn::Id.to_loader_key(&asc, ""),
        EpisodeColumn::Id.to_loader_key(&desc, ""),
        "the same columns in a different argument order should give one loader key",
    );

    tmp_db!(Episode).drop().await
}

// ---------------------------------------------------------------------------
// What gql_look_ahead_all covers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn look_ahead_all_excludes_graphql_skip_col() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Agent {
            pub name: String,
            #[graphql(skip)]
            pub badge_hashed: String,
        }
    }
    use test::*;

    let cols = Agent::gql_look_ahead_all().iter().map(|l| l.c).collect::<Vec<_>>();

    pretty_eq!(
        cols.contains(&"badge_hashed"),
        false,
        "look ahead all should not reach a graphql skipped column",
    );
    pretty_eq!(
        cols.contains(&"name"),
        true,
        "look ahead all should cover the exposed columns"
    );

    tmp_db!(Agent).drop().await
}

#[tokio::test]
async fn look_ahead_cols_keeps_only_the_named_cols() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Episode {
            pub title: String,
            pub season: i32,
        }
    }
    use test::*;

    let cols = Episode::gql_look_ahead_cols(&[EpisodeColumn::Title])
        .iter()
        .map(|l| l.c)
        .collect::<Vec<_>>();

    pretty_eq!(cols, vec!["title"], "look ahead cols should keep only the named column");

    tmp_db!(Episode).drop().await
}
