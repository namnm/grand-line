use grand_line::prelude::*;

#[tokio::test]
async fn has_many_some_none() -> Res<()> {
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

        #[search(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Alias);
    let s = schema_q::<UserSearchQuery>(&tmp.db).finish();

    let u1 = am_create!(User).exec_without_ctx(&tmp.db).await?;
    let u2 = am_create!(User).exec_without_ctx(&tmp.db).await?;
    am_create!(Alias {
        name: "Liv",
        user_id: u1.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = r#"
    query test {
        userSearch(
            filter: { aliases_some: { name: "Liv" } },
        ) {
            id
        }
    }
    "#;
    let expected = value!({
        "userSearch": [{
            "id": u1.id,
        }],
    });
    exec_assert(&s, q, None, &expected).await;

    let q = r#"
    query test {
        userSearch(
            filter: { aliases_none: { name: "Liv" } },
            orderBy: [IdAsc],
        ) {
            id
        }
    }
    "#;
    let expected = value!({
        "userSearch": [{
            "id": u2.id,
        }],
    });
    exec_assert(&s, q, None, &expected).await;

    tmp.drop().await
}

#[tokio::test]
async fn has_many_every() -> Res<()> {
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

        #[search(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Alias);
    let s = schema_q::<UserSearchQuery>(&tmp.db).finish();

    // u1: every alias matches "Liv".
    let u1 = am_create!(User).exec_without_ctx(&tmp.db).await?;
    am_create!(Alias {
        name: "Liv",
        user_id: u1.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(Alias {
        name: "Liv",
        user_id: u1.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    // u2: mixed aliases, not every one matches "Liv".
    let u2 = am_create!(User).exec_without_ctx(&tmp.db).await?;
    am_create!(Alias {
        name: "Liv",
        user_id: u2.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(Alias {
        name: "Fauxlivia",
        user_id: u2.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    // u3: no aliases, vacuously matches every.
    let u3 = am_create!(User).exec_without_ctx(&tmp.db).await?;

    let q = r#"
    query test {
        userSearch(
            filter: { aliases_every: { name: "Liv" } },
            orderBy: [IdAsc],
        ) {
            id
        }
    }
    "#;
    let expected = value!({
        "userSearch": [{
            "id": u1.id,
        }, {
            "id": u3.id,
        }],
    });
    exec_assert(&s, q, None, &expected).await;

    tmp.drop().await
}

#[tokio::test]
async fn many_to_many_some_none() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            #[many_to_many]
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

        #[search(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Org, UserInOrg);
    let s = schema_q::<UserSearchQuery>(&tmp.db).finish();

    let u1 = am_create!(User).exec_without_ctx(&tmp.db).await?;
    let u2 = am_create!(User).exec_without_ctx(&tmp.db).await?;
    let o = am_create!(Org {
        name: "Fringe",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(UserInOrg {
        user_id: u1.id.clone(),
        org_id: o.id,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = r#"
    query test {
        userSearch(
            filter: { orgs_some: { name: "Fringe" } },
        ) {
            id
        }
    }
    "#;
    let expected = value!({
        "userSearch": [{
            "id": u1.id,
        }],
    });
    exec_assert(&s, q, None, &expected).await;

    let q = r#"
    query test {
        userSearch(
            filter: { orgs_none: { name: "Fringe" } }, orderBy: [IdAsc],
        ) {
            id
        }
    }
    "#;
    let expected = value!({
        "userSearch": [{
            "id": u2.id,
        }],
    });
    exec_assert(&s, q, None, &expected).await;

    tmp.drop().await
}

#[tokio::test]
async fn belongs_to_some_none() -> Res<()> {
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

        #[search(Alias)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Alias);
    let s = schema_q::<AliasSearchQuery>(&tmp.db).finish();

    let u1 = am_create!(User {
        name: "Olivia",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let u2 = am_create!(User {
        name: "Peter",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let a1 = am_create!(Alias {
        user_id: u1.id,
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let a2 = am_create!(Alias {
        user_id: u2.id,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = r#"
    query test {
        aliasSearch(
            filter: { user_some: { name: "Olivia" } },
        ) {
            id
        }
    }
    "#;
    let expected = value!({
        "aliasSearch": [{
            "id": a1.id,
        }],
    });
    exec_assert(&s, q, None, &expected).await;

    let q = r#"
    query test {
        aliasSearch(
            filter: { user_none: { name: "Olivia" } },
            orderBy: [IdAsc],
        ) {
            id
        }
    }
    "#;
    let expected = value!({
        "aliasSearch": [{
            "id": a2.id,
        }],
    });
    exec_assert(&s, q, None, &expected).await;

    tmp.drop().await
}
