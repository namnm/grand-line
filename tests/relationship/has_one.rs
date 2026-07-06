use grand_line::prelude::*;

#[tokio::test]
async fn has_one_returns_child() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            #[has_one]
            pub person: Person,
        }
        #[model]
        pub struct Person {
            pub gender: String,
            pub user_id: String,
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Person);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let u = am_create!(User).exec_without_ctx(&tmp.db).await?;
    am_create!(Person {
        gender: "Unknown",
        user_id: u.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            person {
                gender
            }
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "person": {
                "gender": "Unknown",
            },
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
            #[has_one(resolver)]
            pub person: Person,
        }
        #[model]
        pub struct Person {
            pub gender: String,
            pub user_id: String,
        }

        #[one_resolver(Person)]
        fn resolve_person() {
            filter!(Person {
                gender: "Female"
            })
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User, Person);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let u = am_create!(User).exec_without_ctx(&tmp.db).await?;
    am_create!(Person {
        gender: "Male",
        user_id: u.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            person {
                gender
            }
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "person": null,
        },
    });

    exec_assert_id(&s, q, &u.id, &expected).await;
    tmp.drop().await
}
