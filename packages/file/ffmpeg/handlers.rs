use crate::ffmpeg::media_info::*;
use crate::ffmpeg::process::*;
use crate::prelude::*;

/// Default FileHandlers that probes and minifies uploads with the ffprobe/ffmpeg binaries,
/// see the readme, both must be installed on the host system, this crate does not vendor
/// or download them. Plug it in with FileConfig { handlers: Arc::new(FfmpegFileHandlers::default()), .. }.
#[derive(Clone, Default)]
pub struct FfmpegFileHandlers(pub FfmpegConfig);

#[async_trait]
impl FileHandlers for FfmpegFileHandlers {
    async fn on_upload_confirm(&self, ctx: &Context<'_>, file: &FileSql) -> Res<()> {
        // the status flip to Processing runs on the request's own transaction, like any
        // other resolver body, the download/probe/minify/reupload below does not, it
        // outlives the request, so it gets its own raw db connection instead of ctx.tx()
        let tx = &*ctx.tx().await?;
        am_update!(File {
            id: file.id.clone(),
            status: FileStatus::Processing,
        })
        .exec_without_ctx(tx)
        .await?;

        let db = ctx
            .data_opt_impl::<Arc<DatabaseConnection>>()
            .ok_or(CoreGraphQLErr::CtxDb404)?;
        let db = Arc::clone(db);
        let client = Arc::clone(ctx.file_s3_client()?);
        let c = ctx.file_config().clone();
        let cfg = self.0.clone();
        let file = file.clone();

        spawn(async move {
            if let Err(e) = process(&db, &client, &c, &cfg, &file).await {
                let _ = mark_failed(&db, &file.id, &e.to_string()).await;
            }
        });

        Ok(())
    }
}

/// Downloads the object, runs ffprobe, optionally minifies it, reuploads if the minified
/// version is smaller, and moves the row to Ready with the probe output as metadata.
/// ffprobe failing (missing binary, unreadable/non-media content...) does not fail the
/// whole file, it is only used to fill in optional display metadata, not to decide
/// whether the upload itself succeeded.
async fn process(db: &DatabaseConnection, client: &aws_sdk_s3::Client, c: &FileConfig, cfg: &FfmpegConfig, file: &FileSql) -> Res<()> {
    let input = TempPath::new(ext_of(&file.filename));
    download_to(client, &c.bucket, &file.key, input.path()).await?;

    let probe = ffprobe(&cfg.ffprobe_bin, input.path()).await.ok();
    let media = probe.as_ref().map(extract_media_info).unwrap_or_default();

    let is_image = cfg.minify_images && file.content_type.starts_with("image/");
    let is_video = cfg.minify_videos && file.content_type.starts_with("video/");

    let mut size = file.size;
    let mut etag = file.etag.clone();

    if is_image || is_video {
        let output = TempPath::new(ext_of(&file.filename));
        if is_image {
            minify_image(&cfg.ffmpeg_bin, input.path(), output.path(), cfg.max_image_dimension, cfg.image_quality).await?;
        } else {
            minify_video(&cfg.ffmpeg_bin, input.path(), output.path(), cfg.max_video_height, cfg.video_crf).await?;
        }

        let minified_size = tokio::fs::metadata(output.path()).await.map_err(err_s3)?.len();
        if minified_size > 0 && minified_size < file.size.unsigned_abs() {
            etag = upload_from(client, &c.bucket, &file.key, &file.content_type, output.path()).await?;
            size = i64::try_from(minified_size).unwrap_or(file.size);
        }
    }

    am_update!(File {
        id: file.id.clone(),
        size,
        etag,
        metadata: probe,
        media_width: media.width,
        media_height: media.height,
        media_codec: media.codec,
        status: FileStatus::Ready,
    })
    .exec_without_ctx(db)
    .await?;

    Ok(())
}

/// Marks a row Failed with the error message recorded in metadata, best-effort, errors
/// from this are swallowed by the caller since there is no request left to report them to.
async fn mark_failed(db: &DatabaseConnection, id: &str, message: &str) -> Res<()> {
    am_update!(File {
        id: id.to_owned(),
        status: FileStatus::Failed,
        metadata: Some(json!({ "error": message })),
    })
    .exec_without_ctx(db)
    .await?;
    Ok(())
}
