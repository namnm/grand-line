#[path = "./setup.rs"]
mod setup;
use setup::*;
use std::time::Duration;

/// Polls until a row's status leaves Processing, since on_upload_confirm's work runs in a
/// spawned background task rather than being done by the time the request returns.
async fn wait_for_final_status(tmp: &TmpDb, id: &str) -> Res<FileStatus> {
    let mut status = FileStatus::Processing;
    for _ in 0_u8..50 {
        let row = File::find_by_id(id).one(&tmp.db).await?;
        let Some(row) = row else {
            return TestErr::expect("row should still exist while processing");
        };
        status = row.status;
        if status != FileStatus::Processing {
            break;
        }
        sleep(Duration::from_millis(20)).await;
    }
    Ok(status)
}

#[tokio::test]
async fn on_upload_confirm_marks_failed_when_minify_cannot_read_the_object() -> Res<()> {
    let head = mock_head_object_response(17, "image/png", "peter-photo-etag")?;
    let head_req = ::http::Request::builder()
        .uri("http://mock/peter/photo.png")
        .body(aws_smithy_types::body::SdkBody::empty())
        .map_err(err_s3)?;

    let get = ::http::Response::builder()
        .status(200)
        .body(aws_smithy_types::body::SdkBody::from(b"not a real image".to_vec()))
        .map_err(err_s3)?;
    let get_req = ::http::Request::builder()
        .uri("http://mock/peter/photo.png")
        .body(aws_smithy_types::body::SdkBody::empty())
        .map_err(err_s3)?;

    let tmp = tmp_db!(File);
    let c = FileConfig {
        bucket: BUCKET.to_owned(),
        handlers: Arc::new(FfmpegFileHandlers::default()),
        ..Default::default()
    };
    let client = mock_s3_client(vec![ReplayEvent::new(head_req, head), ReplayEvent::new(get_req, get)]);
    let s = schema_qm::<Query, Mutation>(&tmp.db).data(c).data(Arc::new(client)).finish();

    let f = am_create!(File {
        key: "peter/photo.png",
        filename: "photo.png",
        content_type: "image/png",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let v = value!({
        "id": f.id.clone(),
    });
    exec_assert_ok(&s, Q_UPLOAD_CONFIRM, Some(v)).await;

    let status = wait_for_final_status(&tmp, &f.id).await?;
    // ffprobe failing alone does not fail a file (see the test below), an image/video
    // content_type also goes through minify, which cannot make sense of fake bytes either
    // and its failure is fatal, this is what actually drives the row to Failed here
    pretty_eq!(status, FileStatus::Failed, "minify should fail on bytes that are not a real image");

    let row = File::find_by_id(&f.id).one(&tmp.db).await?;
    let Some(row) = row else {
        return TestErr::expect("row should still exist after processing fails");
    };
    pretty_eq!(
        row.metadata.map(|m| m.ptr("/error").as_str().is_some()),
        Some(true),
        "the error should be recorded on the row for later inspection",
    );

    tmp.drop().await
}

#[tokio::test]
async fn on_upload_confirm_ignores_a_failed_probe_for_non_media_content() -> Res<()> {
    let head = mock_head_object_response(17, "application/pdf", "peter-report-etag")?;
    let head_req = ::http::Request::builder()
        .uri("http://mock/peter/report.pdf")
        .body(aws_smithy_types::body::SdkBody::empty())
        .map_err(err_s3)?;

    let get = ::http::Response::builder()
        .status(200)
        .body(aws_smithy_types::body::SdkBody::from(b"not a real pdf".to_vec()))
        .map_err(err_s3)?;
    let get_req = ::http::Request::builder()
        .uri("http://mock/peter/report.pdf")
        .body(aws_smithy_types::body::SdkBody::empty())
        .map_err(err_s3)?;

    let tmp = tmp_db!(File);
    let c = FileConfig {
        bucket: BUCKET.to_owned(),
        handlers: Arc::new(FfmpegFileHandlers::default()),
        ..Default::default()
    };
    let client = mock_s3_client(vec![ReplayEvent::new(head_req, head), ReplayEvent::new(get_req, get)]);
    let s = schema_qm::<Query, Mutation>(&tmp.db).data(c).data(Arc::new(client)).finish();

    let f = am_create!(File {
        key: "peter/report.pdf",
        filename: "report.pdf",
        content_type: "application/pdf",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let v = value!({
        "id": f.id.clone(),
    });
    exec_assert_ok(&s, Q_UPLOAD_CONFIRM, Some(v)).await;

    let status = wait_for_final_status(&tmp, &f.id).await?;
    // a pdf is not image/* or video/*, so minify never runs, ffprobe failing on it (missing
    // binary or unreadable content) must not drag the whole file down to Failed
    pretty_eq!(status, FileStatus::Ready, "a failed probe alone should not fail a non-media file");

    let row = File::find_by_id(&f.id).one(&tmp.db).await?;
    let Some(row) = row else {
        return TestErr::expect("row should still exist after processing");
    };
    pretty_eq!(row.metadata.is_none(), true, "metadata should stay empty when the probe failed");
    pretty_eq!(row.media_width.is_none(), true, "media_width should stay empty when the probe failed");
    pretty_eq!(row.media_height.is_none(), true, "media_height should stay empty when the probe failed");
    pretty_eq!(row.media_codec.is_none(), true, "media_codec should stay empty when the probe failed");

    tmp.drop().await
}
