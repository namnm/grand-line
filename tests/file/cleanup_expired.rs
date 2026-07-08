#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn cleanup_expired_pending_removes_only_expired_pending_rows() -> Res<()> {
    let req1 = ::http::Request::builder()
        .uri("http://mock/walter/never-uploaded.pdf")
        .body(aws_smithy_types::body::SdkBody::empty())
        .map_err(err_s3)?;
    let req2 = ::http::Request::builder()
        .uri("http://mock/walter/abandoned.pdf")
        .body(aws_smithy_types::body::SdkBody::empty())
        .map_err(err_s3)?;
    let events = vec![
        ReplayEvent::new(req1, mock_delete_object_response()?),
        ReplayEvent::new(req2, mock_delete_object_response()?),
    ];
    let d = setup(events).await?;

    let expired_no_object = am_create!(File {
        key: "walter/never-uploaded.pdf",
        filename: "never-uploaded.pdf",
        content_type: "application/pdf",
        upload_expires_at: now() - duration_m(10),
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    let expired_with_object = am_create!(File {
        key: "walter/abandoned.pdf",
        filename: "abandoned.pdf",
        content_type: "application/pdf",
        upload_expires_at: now() - duration_m(10),
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    let still_pending = am_create!(File {
        key: "astrid/in-progress.pdf",
        filename: "in-progress.pdf",
        content_type: "application/pdf",
        upload_expires_at: now() + duration_m(10),
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    let expired_but_uploaded = am_create!(File {
        key: "peter/already-confirmed.pdf",
        filename: "already-confirmed.pdf",
        content_type: "application/pdf",
        status: FileStatus::Uploaded,
        upload_expires_at: now() - duration_m(10),
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    let r = exec_assert_ok(&d.s, Q_CLEANUP_EXPIRED_PENDING, None).await;
    let r = r.data.to_json()?;
    pretty_eq!(
        r.ptr("/fileCleanupExpiredPending"),
        &json!(2),
        "only the two expired pending rows should be cleaned up",
    );

    let remaining_ids = File::find()
        .all(&d.tmp.db)
        .await?
        .into_iter()
        .map(|f| f.id)
        .collect::<Vec<_>>();

    pretty_eq!(
        remaining_ids.contains(&expired_no_object.id),
        false,
        "expired pending row with no object should be removed",
    );
    pretty_eq!(
        remaining_ids.contains(&expired_with_object.id),
        false,
        "expired pending row with an abandoned object should be removed",
    );
    pretty_eq!(
        remaining_ids.contains(&still_pending.id),
        true,
        "pending row that has not expired yet should be kept",
    );
    pretty_eq!(
        remaining_ids.contains(&expired_but_uploaded.id),
        true,
        "already uploaded row should be kept even though its upload window expired",
    );

    d.tmp.drop().await
}
