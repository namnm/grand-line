pub use grand_line::prelude::*;

#[tokio::test]
async fn sql_dep_cols() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            pub first_name: String,
            #[graphql(skip)]
            pub last_name: String,
            #[resolver(sql_dep = "first_name, last_name")]
            pub full_name: String,
        }

        async fn resolve_full_name(u: &UserGql, _ctx: &Context<'_>) -> Res<String> {
            let full_name = [
                u.first_name.clone().ok_or(CoreDbErr::GqlResolverNone)?,
                u.last_name.clone().ok_or(CoreDbErr::GqlResolverNone)?,
            ]
            .join(" ");
            Ok(full_name)
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let u = am_create!(User {
        first_name: "Olivia",
        last_name: "Dunham",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            fullName
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "fullName": "Olivia Dunham",
        },
    });

    exec_assert_id(&s, q, &u.id, &expected).await;
    tmp.drop().await
}

#[tokio::test]
async fn sql_dep_exprs() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            a: i64,
            #[sql_expr(Expr::col(Column::A).add(1000))]
            b: i64,
            #[resolver(sql_dep = "a, b")]
            c: i64,
            #[resolver(custom_d, sql_dep = "a, b")]
            d: i64,
        }

        async fn resolve_c(u: &UserGql, _ctx: &Context<'_>) -> Res<i64> {
            let a = u.a.ok_or(CoreDbErr::GqlResolverNone)?;
            let b = u.b.ok_or(CoreDbErr::GqlResolverNone)?;
            Ok(a + b)
        }
        async fn custom_d(u: &UserGql, ctx: &Context<'_>) -> Res<i64> {
            resolve_c(u, ctx).await
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
            c
            d
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "c": 1002,
            "d": 1002,
        },
    });

    exec_assert_id(&s, q, &u.id, &expected).await;
    tmp.drop().await
}
