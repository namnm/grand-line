pub use grand_line::prelude::*;

#[tokio::test]
async fn has_many_count() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            #[has_many(count)]
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

    let u1 = am_create!(User).exec_without_ctx(&tmp.db).await?;
    let u2 = am_create!(User).exec_without_ctx(&tmp.db).await?;
    am_create!(Alias {
        name: "Liv",
        user_id: u1.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(Alias {
        name: "Fauxlivia",
        user_id: u1.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            aliasesCount
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "aliasesCount": 2,
        },
    });
    exec_assert_id(&s, q, &u1.id, &expected).await;

    let expected = value!({
        "userDetail": {
            "aliasesCount": 0,
        },
    });
    exec_assert_id(&s, q, &u2.id, &expected).await;

    let q = r#"
    query test($id: ID!) {
        userDetail(id: $id) {
            aliasesCount(
                filter: { name: "Liv" },
            )
        }
    }
    "#;
    let expected = value!({
        "userDetail": {
            "aliasesCount": 1,
        },
    });
    exec_assert_id(&s, q, &u1.id, &expected).await;

    tmp.drop().await
}

#[tokio::test]
async fn has_many_count_custom_resolver() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            #[has_many(count, count_resolver)]
            pub aliases: Alias,
        }
        #[model]
        pub struct Alias {
            pub name: String,
            pub user_id: String,
        }

        #[count_resolver(Alias)]
        fn resolve_aliases_count() {
            let f = filter!(Alias {
                name: "Walter"
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
        name: "Walter",
        user_id: u.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(Alias {
        name: "Astrid",
        user_id: u.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            aliasesCount
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "aliasesCount": 1,
        },
    });
    exec_assert_id(&s, q, &u.id, &expected).await;

    tmp.drop().await
}

#[tokio::test]
async fn has_many_count_custom_resolver_uses_parent_field() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            pub cover_identity: String,
            #[has_many(count, count_resolver = "count_aliases_matching_cover_identity")]
            pub aliases: Alias,
        }
        #[model]
        pub struct Alias {
            pub name: String,
            pub user_id: String,
        }

        #[count_resolver(Alias, parent = "User")]
        fn count_aliases_matching_cover_identity() {
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
            aliasesCount
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "coverIdentity": "Bell",
            "aliasesCount": 1,
        },
    });
    exec_assert_id(&s, q, &u.id, &expected).await;

    tmp.drop().await
}

#[tokio::test]
async fn many_to_many_count() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            #[many_to_many(count)]
            pub orgs: Org,
        }
        #[model]
        pub struct Org {
            pub name: String,
        }
        #[model]
        pub struct UserInOrg {
            pub user_id: String,
            pub org_id: String,
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Org, UserInOrg);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let u = am_create!(User).exec_without_ctx(&tmp.db).await?;
    let o1 = am_create!(Org {
        name: "Fringe",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let o2 = am_create!(Org {
        name: "FBI",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(UserInOrg {
        user_id: u.id.clone(),
        org_id: o1.id,
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(UserInOrg {
        user_id: u.id.clone(),
        org_id: o2.id,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            orgsCount
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "orgsCount": 2,
        },
    });
    exec_assert_id(&s, q, &u.id, &expected).await;

    let q = r#"
    query test($id: ID!) {
        userDetail(id: $id) {
            orgsCount(
                filter: { name: "Fringe" },
            )
        }
    }
    "#;
    let expected = value!({
        "userDetail": {
            "orgsCount": 1,
        },
    });
    exec_assert_id(&s, q, &u.id, &expected).await;

    tmp.drop().await
}
