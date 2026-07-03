pub use grand_line::prelude::*;

#[tokio::test]
async fn resolver_default_name() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            #[many_to_many(resolver)]
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

        #[many_resolver(Org)]
        fn resolve_orgs() {
            let f = filter!(Org {
                name: "Fringe"
            });
            f.into()
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
            orgs {
                name
            }
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "orgs": [{
                "name": "Fringe",
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
            #[many_to_many(resolver = "custom_orgs")]
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

        #[many_resolver(Org)]
        fn custom_orgs() {
            order_by!(Org[NameDesc]).into()
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
        name: "Fringe Division",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let o2 = am_create!(Org {
        name: "Massive Dynamic",
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
            orgs {
                name
            }
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "orgs": [{
                "name": "Massive Dynamic",
            }, {
                "name": "Fringe Division",
            }],
        },
    });

    exec_assert_id(&s, q, &u.id, &expected).await;
    tmp.drop().await
}

#[tokio::test]
async fn many_to_many_returns_related() -> Res<()> {
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

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Org, UserInOrg);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let u = am_create!(User).exec_without_ctx(&tmp.db).await?;
    let o = am_create!(Org {
        name: "Fringe",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(UserInOrg {
        user_id: u.id.clone(),
        org_id: o.id,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            orgs {
                name
            }
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "orgs": [{
                "name": "Fringe",
            }],
        },
    });

    exec_assert_id(&s, q, &u.id, &expected).await;
    tmp.drop().await
}
