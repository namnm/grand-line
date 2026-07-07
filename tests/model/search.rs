use grand_line::prelude::*;

// ---------------------------------------------------------------------------
// Search resolver
// ---------------------------------------------------------------------------

// search resolver returns all records and supports pagination.
#[tokio::test]
async fn returns_all() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            pub name: String,
        }

        #[search(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User);
    let s = schema_q::<UserSearchQuery>(&tmp.db).finish();

    am_create!(User {
        name: "Olivia",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(User {
        name: "Peter",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test {
        userSearch {
            name
        }
    }
    ";
    let r = exec_assert_ok(&s, q, None).await;
    let r = r.data.to_json()?;

    pretty_eq!(r.arr("/userSearch").len(), 2, "records length should be 2");

    tmp.drop().await
}

// search resolver respects page limit.
#[tokio::test]
async fn pagination_limit() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            pub name: String,
        }

        #[search(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User);
    let s = schema_q::<UserSearchQuery>(&tmp.db).finish();

    am_create!(User {
        name: "Olivia",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(User {
        name: "Peter",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(User {
        name: "Walter",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test {
        userSearch(page: { limit: 1, offset: 0 }) {
            name
        }
    }
    ";
    let r = exec_assert_ok(&s, q, None).await;
    let r = r.data.to_json()?;

    pretty_eq!(r.arr("/userSearch").len(), 1, "page limit should restrict to 1 record");

    tmp.drop().await
}

// ---------------------------------------------------------------------------
// Count resolver
// ---------------------------------------------------------------------------

// count resolver returns the correct count.
#[tokio::test]
async fn count() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            pub name: String,
        }

        #[count(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User);
    let s = schema_q::<UserCountQuery>(&tmp.db).finish();

    am_create!(User {
        name: "Olivia",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(User {
        name: "Peter",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test {
        userCount
    }
    ";
    let expected = value!({
        "userCount": 2,
    });
    exec_assert(&s, q, None, &expected).await;

    tmp.drop().await
}
