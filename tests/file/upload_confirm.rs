#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn upload_confirm_moves_pending_row_to_uploaded() -> Res<()> {
    let head = mock_head_object_response(42, "video/mp4", "olivia-scan-etag")?;
    let req = ::http::Request::builder()
        .uri("http://mock/olivia/scan.mp4")
        .body(aws_smithy_types::body::SdkBody::empty())
        .map_err(err_s3)?;
    let d = setup(vec![ReplayEvent::new(req, head)]).await?;

    let f = am_create!(File {
        key: "olivia/scan.mp4",
        filename: "scan.mp4",
        content_type: "application/octet-stream",
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    let v = value!({
        "id": f.id,
    });
    let r = exec_assert_ok(&d.s, Q_UPLOAD_CONFIRM, Some(v)).await;
    let r = r.data.to_json()?;

    pretty_eq!(r.str("/fileUploadConfirm/status"), "UPLOADED", "status should move to uploaded");
    pretty_eq!(r.ptr("/fileUploadConfirm/size"), &json!(42), "size should come from the head response");
    pretty_eq!(
        r.str("/fileUploadConfirm/contentType"),
        "video/mp4",
        "content_type should come from the head response",
    );
    pretty_eq!(
        r.str("/fileUploadConfirm/etag"),
        "\"olivia-scan-etag\"",
        "etag should come from the head response, s3 etags are quoted",
    );

    let download_url = r.str("/fileUploadConfirm/downloadUrl");
    pretty_eq!(download_url.is_empty(), false, "downloadUrl should be a presigned url once uploaded");

    let updated = File::find_by_id(&f.id).one(&d.tmp.db).await?;
    let Some(updated) = updated else {
        return TestErr::expect("file row should still exist after confirm");
    };
    pretty_eq!(
        updated.metadata,
        Some(json!({ "processed_by": "mock_file_handlers" })),
        "on_upload_confirm handler should have run and written metadata",
    );

    d.tmp.drop().await
}
