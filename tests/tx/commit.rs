#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// A dataloader in the request must not keep the transaction from committing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mutation_selecting_a_relation_still_commits() -> Res<()> {
    let d = setup().await?;

    let q = "
    mutation test($data: InvestigationCreate!) {
        investigationCreate(data: $data) {
            id
            name
            reports {
                title
            }
        }
    }
    ";
    let v = value!({
        "data": {
            "name": "The Pattern",
        },
    });

    let r = exec_assert_ok(&d.s, q, Some(v)).await;
    let r = r.data.to_json()?;
    let id = r.str("/investigationCreate/id");

    let row = Investigation::find_by_id(id).one(&d.tmp.db).await?;
    let Some(row) = row else {
        return TestErr::expect("row should be committed even though the response used a dataloader");
    };
    pretty_eq!(row.name, "The Pattern", "committed row should hold the created name");

    d.tmp.drop().await
}

#[tokio::test]
async fn query_selecting_a_relation_returns_the_relation_rows() -> Res<()> {
    let d = setup().await?;

    let i = am_create!(Investigation {
        name: "Jacksonville",
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;
    am_create!(Report {
        title: "Olivia",
        investigation_id: i.id.clone(),
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    let q = "
    query test($id: ID!) {
        investigationDetail(id: $id) {
            reports {
                title
            }
        }
    }
    ";
    let expected = value!({
        "investigationDetail": {
            "reports": [
                {
                    "title": "Olivia",
                },
            ],
        },
    });

    exec_assert_id(&d.s, q, &i.id, &expected).await;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// Only a mutation opens a transaction
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_query_reads_from_the_pool_without_a_transaction() -> Res<()> {
    let d = setup().await?;

    let q = "
    query {
        investigationConnIsTx
    }
    ";
    let expected = value!({
        "investigationConnIsTx": false,
    });

    exec_assert(&d.s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn a_query_can_still_force_a_transaction_open() -> Res<()> {
    let d = setup().await?;

    let q = "
    query {
        investigationForcedTxIsTx
    }
    ";
    let expected = value!({
        "investigationForcedTxIsTx": true,
    });

    exec_assert(&d.s, q, None, &expected).await;

    d.tmp.drop().await
}
