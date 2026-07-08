#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn upload_init_creates_pending_row_with_upload_url() -> Res<()> {
    let d = setup(vec![]).await?;

    let v = value!({
        "data": {
            "filename": "walter-notes.pdf",
            "contentType": "application/pdf",
        },
    });
    let r = exec_assert_ok(&d.s, Q_UPLOAD_INIT, Some(v)).await;
    let r = r.data.to_json()?;

    let upload_url = r.str("/fileUploadInit/uploadUrl");
    pretty_eq!(upload_url.is_empty(), false, "upload_url should be returned");

    let key = r.str("/fileUploadInit/inner/key");
    pretty_eq!(key, "olivia/walter-notes.pdf", "key should use the handler's key builder");
    pretty_eq!(
        r.str("/fileUploadInit/inner/status"),
        "PENDING",
        "status should start as pending",
    );
    pretty_eq!(
        r.str("/fileUploadInit/inner/contentType"),
        "application/pdf",
        "content_type should match what the client sent",
    );

    let id = r.str("/fileUploadInit/inner/id");
    let f = File::find_by_id(id).one(&d.tmp.db).await?;
    let Some(f) = f else {
        return TestErr::expect("file row should exist after fileUploadInit");
    };
    pretty_eq!(f.key, key, "row key should match the response key");
    pretty_eq!(f.status, FileStatus::Pending, "row status should be pending");

    d.tmp.drop().await
}
