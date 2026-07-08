use crate::prelude::*;
use aws_sdk_s3::presigning::PresigningConfig;

// ---------------------------------------------------------------------------
// File model
// ---------------------------------------------------------------------------

/// Lifecycle of a File row, driven by the upload/process chain, see the readme.
#[sql_enum]
pub enum FileStatus {
    /// Row created, presigned upload url issued, no object in the bucket yet.
    Pending,
    /// fileUploadConfirm verified the object exists in the bucket.
    Uploaded,
    /// A FileHandlers::on_upload_confirm implementation is transforming the file.
    Processing,
    /// Processing finished successfully.
    Ready,
    /// Processing failed.
    Failed,
}

/// A file stored in an s3/r2 compatible bucket, see the readme for the upload chain.
#[model]
pub struct File {
    /// Object key/path inside the configured bucket.
    pub key: String,
    /// Original filename provided by the client on fileUploadInit.
    pub filename: String,
    pub content_type: String,
    #[default(0)]
    pub size: i64,
    pub etag: Option<String>,
    #[default(FileStatus::Pending)]
    pub status: FileStatus,
    /// When the presigned upload url from fileUploadInit stops being valid, used by
    /// fileCleanupExpiredPending to garbage collect rows the client never confirmed.
    #[default(now())]
    pub upload_expires_at: DateTimeUtc,
    /// Plain foreign key, not a belongs_to relation, since this crate does not own the
    /// host's Org model, see the readme for wiring a has_many back from Org.
    pub org_id: Option<String>,
    /// Host-written processing output (raw ffprobe json...), set by a
    /// FileHandlers::on_upload_confirm implementation.
    #[graphql(skip)]
    pub metadata: Option<JsonValue>,
    /// Pixel width of the first video/image stream, from ffprobe, when available, so
    /// clients do not have to parse metadata just to read the dimensions.
    pub media_width: Option<i64>,
    /// Pixel height of the first video/image stream, from ffprobe, when available.
    pub media_height: Option<i64>,
    /// Codec name of the first video/image stream (e.g. "h264", "mjpeg"), from ffprobe,
    /// when available.
    pub media_codec: Option<String>,
    /// Presigned get url, computed per request, None while status is Pending.
    #[resolver(sql_dep = "key, status")]
    pub download_url: Option<String>,
}

async fn resolve_download_url(f: &FileGql, ctx: &Context<'_>) -> Res<Option<String>> {
    let key = f.key.clone().ok_or(CoreDbErr::GqlResolverNone)?;
    let status = f.status.ok_or(CoreDbErr::GqlResolverNone)?;
    if status == FileStatus::Pending {
        return Ok(None);
    }

    let client = ctx.file_s3_client()?;
    let c = ctx.file_config();
    let presign = PresigningConfig::expires_in(std_duration_ms(c.download_url_expires_ms)).map_err(err_s3)?;

    let req = client
        .get_object()
        .bucket(&c.bucket)
        .key(&key)
        .presigned(presign)
        .await
        .map_err(err_s3)?;

    Ok(Some(req.uri().to_owned()))
}

// ---------------------------------------------------------------------------
// File with a presigned upload url
// ---------------------------------------------------------------------------

/// Returned by fileUploadInit, the client PUTs the raw bytes to upload_url directly.
pub struct FileWithUploadUrl {
    pub inner: FileSql,
    pub upload_url: String,
}
#[Object]
impl FileWithUploadUrl {
    pub async fn upload_url(&self) -> String {
        self.upload_url.clone()
    }
    pub async fn inner(&self, ctx: &Context<'_>) -> Res<FileGql> {
        let r = self.inner.clone().into_gql(ctx).await?;
        Ok(r)
    }
}
