use crate::prelude::*;

/// Finds Pending rows whose presigned upload url has expired, meaning the client never
/// called fileUploadConfirm, best-effort removes the object from the bucket in case
/// something was actually uploaded without confirming, and permanently deletes the rows.
/// This package ships no scheduler, call this from your own cron job, see the readme.
#[mutation]
fn file_cleanup_expired_pending() -> u64 {
    let c = ctx.file_config();
    let client = ctx.file_s3_client()?;

    let f = filter!(File {
        status: FileStatus::Pending,
        upload_expires_at_lt: now(),
    });
    let rows = f.clone().into_select().all(tx).await?;

    for r in &rows {
        let _ = client.delete_object().bucket(&c.bucket).key(&r.key).send().await;
    }

    File::delete_many().filter(f).exec(tx).await?.rows_affected
}
