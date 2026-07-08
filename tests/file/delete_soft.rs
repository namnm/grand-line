#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn delete_without_permanent_soft_deletes_and_keeps_the_object() -> Res<()> {
    // no events queued, if the resolver wrongly called s3 on a soft delete this would error
    let d = setup(vec![]).await?;

    let f = am_create!(File {
        key: "astrid/report.pdf",
        filename: "report.pdf",
        content_type: "application/pdf",
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    let v = value!({
        "id": f.id.clone(),
        "permanent": null,
    });
    exec_assert_ok(&d.s, Q_DELETE, Some(v)).await;

    let r = File::find_by_id(&f.id).include_deleted(true).one(&d.tmp.db).await?;
    let Some(r) = r else {
        return TestErr::expect("soft deleted row should still exist in the db");
    };
    pretty_eq!(r.deleted_at.is_some(), true, "row should be marked deleted");

    d.tmp.drop().await
}
