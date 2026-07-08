use crate::prelude::*;
use aws_sdk_s3::presigning::PresigningConfig;

/// Input for fileUploadInit, filename and content_type describe the object about to be
/// uploaded, org_id optionally scopes the file, see the readme for the has_many wiring.
#[gql_input]
pub struct FileUploadInit {
    pub filename: String,
    pub content_type: String,
    pub org_id: Option<String>,
}

/// Creates a Pending File row and returns a presigned put url, the client uploads the raw
/// bytes to upload_url directly, then calls fileUploadConfirm once that finishes.
#[mutation]
fn file_upload_init(data: FileUploadInit) -> FileWithUploadUrl {
    let c = ctx.file_config();
    let client = ctx.file_s3_client()?;

    let key = c.handlers.key(ctx, &data.filename).await?;

    let f = am_create!(File {
        key: key.clone(),
        filename: data.filename,
        content_type: data.content_type,
        org_id: data.org_id,
        upload_expires_at: now() + duration_ms(c.upload_url_expires_ms),
    })
    .exec_without_ctx(tx)
    .await?;

    let presign = PresigningConfig::expires_in(std_duration_ms(c.upload_url_expires_ms)).map_err(err_s3)?;
    let req = client
        .put_object()
        .bucket(&c.bucket)
        .key(&key)
        .content_type(f.content_type.clone())
        .presigned(presign)
        .await
        .map_err(err_s3)?;

    FileWithUploadUrl {
        inner: f,
        upload_url: req.uri().to_owned(),
    }
}
