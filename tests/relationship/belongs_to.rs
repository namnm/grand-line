use grand_line::prelude::*;

#[tokio::test]
async fn belongs_to_returns_parent() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            pub name: String,
        }
        #[model]
        pub struct Alias {
            pub user_id: String,
            #[belongs_to]
            pub user: User,
        }

        #[detail(Alias)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Alias);
    let s = schema_q::<AliasDetailQuery>(&tmp.db).finish();

    let u = am_create!(User {
        name: "Olivia",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let f = am_create!(Alias {
        user_id: u.id,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        aliasDetail(id: $id) {
            user {
                name
            }
        }
    }
    ";
    let expected = value!({
        "aliasDetail": {
            "user": {
                "name": "Olivia",
            },
        },
    });

    exec_assert_id(&s, q, &f.id, &expected).await;
    tmp.drop().await
}

#[tokio::test]
async fn resolver_custom_fn() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            pub name: String,
        }
        #[model]
        pub struct Alias {
            pub user_id: String,
            #[belongs_to(resolver)]
            pub user: User,
        }

        #[one_resolver(User)]
        fn resolve_user() {
            filter!(User {
                name: "Walter"
            })
        }

        #[detail(Alias)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Alias);
    let s = schema_q::<AliasDetailQuery>(&tmp.db).finish();

    let u = am_create!(User {
        name: "Olivia",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let f = am_create!(Alias {
        user_id: u.id,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        aliasDetail(id: $id) {
            user {
                name
            }
        }
    }
    ";
    let expected = value!({
        "aliasDetail": {
            "user": null,
        },
    });

    exec_assert_id(&s, q, &f.id, &expected).await;
    tmp.drop().await
}
