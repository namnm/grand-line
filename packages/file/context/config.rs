use crate::prelude::*;

// ---------------------------------------------------------------------------
// File configuration
// ---------------------------------------------------------------------------

/// Runtime configuration for the file package, bucket name, presigned url
/// expiries, and pluggable handlers. The s3 client itself is not part of
/// this config, it is injected on the schema separately, see FileS3Context.
#[derive(Clone)]
pub struct FileConfig {
    pub bucket: String,
    pub upload_url_expires_ms: i64,
    pub download_url_expires_ms: i64,
    pub handlers: Arc<dyn FileHandlers>,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            upload_url_expires_ms: FILE_UPLOAD_URL_EXPIRES_MS,
            download_url_expires_ms: FILE_DOWNLOAD_URL_EXPIRES_MS,
            handlers: Arc::new(DefaultHandlers),
        }
    }
}

// ---------------------------------------------------------------------------
// Pluggable behavior hooks
// ---------------------------------------------------------------------------

/// Extension points for customizing file behavior, all methods have permissive no-op defaults.
#[allow(unused_variables)]
#[async_trait]
pub trait FileHandlers
where
    Self: Send + Sync,
{
    /// Builds the storage key for a newly initiated upload, called from fileUploadInit,
    /// the default is a random ulid segment followed by the original filename.
    async fn key(&self, ctx: &Context<'_>, filename: &str) -> Res<String> {
        Ok(format!("{}/{filename}", ulid()))
    }

    /// Called after fileUploadConfirm verifies the object exists in the bucket and moves
    /// the row to Uploaded. The default is a no-op, override to inspect or transform the
    /// file (ffprobe, image/video compression, thumbnailing...) and move status to
    /// Processing/Ready/Failed once that work is done, outside of this request.
    async fn on_upload_confirm(&self, ctx: &Context<'_>, file: &FileSql) -> Res<()> {
        Ok(())
    }
}

struct DefaultHandlers;
#[async_trait]
impl FileHandlers for DefaultHandlers {
}
