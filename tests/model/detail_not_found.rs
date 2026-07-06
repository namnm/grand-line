use grand_line::prelude::*;

// detail resolver returns null when the record does not exist.
#[tokio::test]
async fn returns_null_when_record_not_found() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            pub name: String,
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            name
        }
    }
    ";
    let expected = value!({
        "userDetail": null,
    });
    exec_assert_id(&s, q, "nonexistent-id", &expected).await;

    tmp.drop().await
}
