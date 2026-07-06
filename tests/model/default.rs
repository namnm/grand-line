use grand_line::prelude::*;

#[tokio::test]
async fn insert_defaults() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            #[default("I love you")]
            pub a: String,
            #[default(3000)]
            pub b: i64,
            #[default(9999)]
            pub c: i64,
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let u = am_create!(User {
        c: 9,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    pretty_eq!(u.a, "I love you", "u.a should use its default value");
    pretty_eq!(u.b, 3000, "u.b should use its default value");
    pretty_eq!(u.c, 9, "u.c should keep the explicitly provided value, not the default");

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            a
            b
            c
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "a": "I love you",
            "b": 3000,
            "c": 9,
        },
    });

    exec_assert_id(&s, q, &u.id, &expected).await;
    tmp.drop().await
}
