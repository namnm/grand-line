use grand_line::prelude::*;

#[tokio::test]
async fn sql_expr_computes_derived_column() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            pub a: i64,
            #[sql_expr(Expr::col(Column::A).add(1000))]
            pub b: i64,
            #[sql_expr(expr_c())]
            pub c: i64,
        }

        fn expr_c() -> SimpleExpr {
            Expr::col(UserColumn::A).add(2000)
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let u = am_create!(User {
        a: 1,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            b
            c
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "b": 1001,
            "c": 2001,
        },
    });

    exec_assert_id(&s, q, &u.id, &expected).await;
    tmp.drop().await
}
