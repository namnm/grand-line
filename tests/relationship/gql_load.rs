use grand_line::prelude::*;

// ---------------------------------------------------------------------------
// gql_load from a scalar resolver, where ctx describes no selection of the
// loaded entity
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gql_load_with_all_loads_a_full_row_from_a_scalar_resolver() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Universe {
            pub name: String,
            pub side: String,
        }
        #[model]
        pub struct Agent {
            pub universe_id: String,
            #[resolver(sql_dep = "universe_id")]
            pub posting: String,
        }

        async fn resolve_posting(a: &AgentGql, ctx: &Context<'_>) -> Res<String> {
            let id = a.universe_id.clone().ok_or(CoreDbErr::GqlResolverNone)?;
            let db = &ctx.db().await?;

            // ctx sits on a scalar field here, so the columns have to be named
            let u = Universe::gql_load_with(
                ctx,
                db,
                UniverseColumn::Id,
                id,
                None,
                None,
                None,
                Universe::gql_look_ahead_all(),
            )
            .await?
            .ok_or(CoreDbErr::Db404)?;

            let name = u.name.ok_or(CoreDbErr::GqlResolverNone)?;
            let side = u.side.ok_or(CoreDbErr::GqlResolverNone)?;
            Ok(format!("{name} / {side}"))
        }

        #[detail(Agent)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(Universe, Agent);
    let s = schema_q::<AgentDetailQuery>(&tmp.db).finish();

    let u = am_create!(Universe {
        name: "Over There",
        side: "red",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let a = am_create!(Agent {
        universe_id: u.id,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        agentDetail(id: $id) {
            posting
        }
    }
    ";
    let expected = value!({
        "agentDetail": {
            "posting": "Over There / red",
        },
    });

    exec_assert_id(&s, q, &a.id, &expected).await;
    tmp.drop().await
}

#[tokio::test]
async fn gql_load_errors_from_a_scalar_resolver() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Universe {
            pub name: String,
        }
        #[model]
        pub struct Agent {
            pub universe_id: String,
            #[resolver(sql_dep = "universe_id")]
            pub posting: String,
        }

        async fn resolve_posting(a: &AgentGql, ctx: &Context<'_>) -> Res<String> {
            let id = a.universe_id.clone().ok_or(CoreDbErr::GqlResolverNone)?;
            let db = &ctx.db().await?;

            // the ctx-driven variant cannot work here, every column would be none
            let u = Universe::gql_load(ctx, db, UniverseColumn::Id, id, None, None, None)
                .await?
                .ok_or(CoreDbErr::Db404)?;

            u.name.ok_or(CoreDbErr::GqlResolverNone.into())
        }

        #[detail(Agent)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(Universe, Agent);
    let s = schema_q::<AgentDetailQuery>(&tmp.db).finish();

    let u = am_create!(Universe {
        name: "Over There",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let a = am_create!(Agent {
        universe_id: u.id,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        agentDetail(id: $id) {
            posting
        }
    }
    ";

    // a server side misuse, so the client sees it masked as internal server
    exec_assert_err_id(&s, q, &a.id, &CoreGraphQLErr::InternalServer).await?;
    tmp.drop().await
}

// ---------------------------------------------------------------------------
// The relation path, where ctx does describe the loaded entity
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gql_load_selects_only_the_join_key_for_typename() -> Res<()> {
    mod test {
        use super::*;

        #[model]
        pub struct Universe {
            pub name: String,
        }
        #[model]
        pub struct Agent {
            pub universe_id: String,
            #[belongs_to]
            pub universe: Universe,
        }

        #[detail(Agent)]
        fn resolver() {
        }
    }
    use test::*;

    let tmp = tmp_db!(Universe, Agent);
    let s = schema_q::<AgentDetailQuery>(&tmp.db).finish();

    let u = am_create!(Universe {
        name: "Over There",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let a = am_create!(Agent {
        universe_id: u.id,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    // a selection set naming no column of Universe is still a selection set, so this
    // stays on the ctx-driven path and must not be mistaken for a scalar resolver
    let q = "
    query test($id: ID!) {
        agentDetail(id: $id) {
            universe {
                __typename
            }
        }
    }
    ";
    let expected = value!({
        "agentDetail": {
            "universe": {
                "__typename": "Universe",
            },
        },
    });

    exec_assert_id(&s, q, &a.id, &expected).await;
    tmp.drop().await
}
