pub use grand_line::prelude::*;

#[tokio::test]
async fn resolver_default_name() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            #[has_many(resolver)]
            pub aliases: Alias,
        }
        #[model]
        pub struct Alias {
            pub name: String,
            pub user_id: String,
        }

        #[many_resolver(Alias)]
        fn resolve_aliases() {
            let f = filter!(Alias {
                name: "Liv"
            });
            f.into()
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Alias);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let u = am_create!(User).exec_without_ctx(&tmp.db).await?;
    am_create!(Alias {
        name: "Liv",
        user_id: u.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(Alias {
        name: "Fauxlivia",
        user_id: u.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            aliases {
                name
            }
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "aliases": [{
                "name": "Liv",
            }],
        },
    });

    exec_assert_id(&s, q, &u.id, &expected).await;
    tmp.drop().await
}

#[tokio::test]
async fn resolver_custom_fn() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            #[has_many(resolver = "custom_aliases")]
            pub aliases: Alias,
        }
        #[model]
        pub struct Alias {
            pub name: String,
            pub user_id: String,
        }

        #[many_resolver(Alias)]
        fn custom_aliases() {
            order_by!(Alias[NameDesc]).into()
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Alias);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let u = am_create!(User).exec_without_ctx(&tmp.db).await?;
    am_create!(Alias {
        name: "Astrid",
        user_id: u.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(Alias {
        name: "Walter",
        user_id: u.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            aliases {
                name
            }
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "aliases": [{
                "name": "Walter",
            }, {
                "name": "Astrid",
            }],
        },
    });

    exec_assert_id(&s, q, &u.id, &expected).await;
    tmp.drop().await
}

#[tokio::test]
async fn resolver_custom_fn_uses_parent_field() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            pub cover_identity: String,
            #[has_many(resolver = "aliases_matching_cover_identity")]
            pub aliases: Alias,
        }
        #[model]
        pub struct Alias {
            pub name: String,
            pub user_id: String,
        }

        #[many_resolver(Alias, parent = "User")]
        fn aliases_matching_cover_identity() {
            filter!(Alias {
                name: parent.cover_identity.clone().unwrap_or_default()
            })
            .into()
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Alias);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let u = am_create!(User {
        cover_identity: "Bell",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(Alias {
        name: "Bell",
        user_id: u.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(Alias {
        name: "Bishop",
        user_id: u.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            coverIdentity
            aliases {
                name
            }
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "coverIdentity": "Bell",
            "aliases": [{
                "name": "Bell",
            }],
        },
    });

    exec_assert_id(&s, q, &u.id, &expected).await;
    tmp.drop().await
}

#[tokio::test]
async fn has_many_returns_children() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            #[has_many]
            pub aliases: Alias,
        }
        #[model]
        pub struct Alias {
            pub name: String,
            pub user_id: String,
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Alias);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let u = am_create!(User).exec_without_ctx(&tmp.db).await?;
    am_create!(Alias {
        name: "Liv",
        user_id: u.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            aliases {
                name
            }
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "aliases": [{
                "name": "Liv",
            }],
        },
    });

    exec_assert_id(&s, q, &u.id, &expected).await;
    tmp.drop().await
}
