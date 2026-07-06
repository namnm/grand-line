use grand_line::prelude::*;

#[tokio::test]
async fn name_override() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            #[graphql(name = "y")]
            pub x: i64,
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let u = am_create!(User {
        x: 42,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            y
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "y": 42,
        },
    });

    exec_assert_id(&s, q, &u.id, &expected).await;

    tmp.drop().await
}

#[tokio::test]
async fn skip() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            pub visible: i64,
            #[graphql(skip)]
            pub hidden: i64,
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();
    let sdl = s.sdl();

    pretty_eq!(sdl.contains("visible"), true, "visible should be in sdl: {sdl}");
    pretty_eq!(sdl.contains("hidden"), false, "hidden should not be in sdl: {sdl}");

    tmp.drop().await
}

#[tokio::test]
async fn doc_comment() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            /// This is a description.
            pub x: i64,
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();
    let sdl = s.sdl();

    pretty_eq!(
        sdl.contains("This is a description."),
        true,
        "sdl should contain the doc comment: {sdl}",
    );

    tmp.drop().await
}

#[tokio::test]
async fn deprecation() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            #[graphql(deprecation = "use y instead")]
            pub x: i64,
            pub y: i64,
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();
    let sdl = s.sdl();

    pretty_eq!(
        sdl.contains("@deprecated"),
        true,
        "sdl should contain the deprecated directive: {sdl}",
    );
    pretty_eq!(
        sdl.contains("use y instead"),
        true,
        "sdl should contain the deprecation reason: {sdl}",
    );

    tmp.drop().await
}

#[tokio::test]
async fn name_override_with_extra() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct User {
            #[graphql(name = "y", deprecation = "should not use")]
            pub x: i64,
        }

        #[detail(User)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(User);
    let s = schema_q::<UserDetailQuery>(&tmp.db).finish();

    let u = am_create!(User {
        x: 7,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        userDetail(id: $id) {
            y
        }
    }
    ";
    let expected = value!({
        "userDetail": {
            "y": 7,
        },
    });

    exec_assert_id(&s, q, &u.id, &expected).await;

    let sdl = s.sdl();
    pretty_eq!(
        sdl.contains("@deprecated"),
        true,
        "sdl should contain the deprecated directive: {sdl}",
    );

    tmp.drop().await
}
