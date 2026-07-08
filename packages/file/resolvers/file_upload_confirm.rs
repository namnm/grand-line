use crate::prelude::*;

/// Verifies the object exists in the bucket (the client-reported size/type on init is never
/// trusted), moves the row to Uploaded with the real size/content_type/etag, and invokes
/// FileHandlers::on_upload_confirm for any further processing.
#[mutation]
fn file_upload_confirm(id: String) -> FileGql {
    let f = File::find_by_id(&id).one_or_404(tx).await?;
    if f.status != FileStatus::Pending {
        return Err(MyErr::UploadNotPending {
            id: id.clone(),
        }
        .into());
    }

    let c = ctx.file_config();
    let client = ctx.file_s3_client()?;

    let head = client
        .head_object()
        .bucket(&c.bucket)
        .key(&f.key)
        .send()
        .await
        .map_err(err_s3)?;

    let size = head.content_length().unwrap_or_default();
    let content_type = head.content_type().map(ToOwned::to_owned).unwrap_or(f.content_type);
    let etag = head.e_tag().map(ToOwned::to_owned);

    let f = am_update!(File {
        id: id.clone(),
        size,
        content_type,
        etag,
        status: FileStatus::Uploaded,
    })
    .exec_without_ctx(tx)
    .await?;

    c.handlers.on_upload_confirm(ctx, &f).await?;

    f.into_gql(ctx).await?
}
