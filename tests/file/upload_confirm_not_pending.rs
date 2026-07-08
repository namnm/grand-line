#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn upload_confirm_rejects_a_row_that_is_not_pending() -> Res<()> {
    let d = setup(vec![]).await?;

    let f = am_create!(File {
        key: "peter/journal.txt",
        filename: "journal.txt",
        content_type: "text/plain",
        status: FileStatus::Uploaded,
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    let v = value!({
        "id": f.id.clone(),
    });
    exec_assert_err(
        &d.s,
        Q_UPLOAD_CONFIRM,
        Some(v),
        &FileErr::UploadNotPending {
            id: f.id,
        },
    )
    .await?;

    d.tmp.drop().await
}
