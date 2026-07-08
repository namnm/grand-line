use crate::prelude::*;

/// Deletes a File row, on a permanent delete the object is also removed from the bucket
/// first, a soft delete leaves the object in place so it stays recoverable.
#[delete(File)]
fn resolver() {
    if permanent.unwrap_or_default() {
        let f = File::find_by_id(&id).one_or_404(tx).await?;
        let c = ctx.file_config();
        let client = ctx.file_s3_client()?;

        client
            .delete_object()
            .bucket(&c.bucket)
            .key(&f.key)
            .send()
            .await
            .map_err(err_s3)?;
    }
}
