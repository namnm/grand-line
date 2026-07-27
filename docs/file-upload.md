# File uploads

`grand_line_file` provides a `File` model backed by an s3/r2-compatible bucket, presigned upload/download urls, and a pluggable processing hook.

## Setup

```rs
#[derive(Default, MergedObject)]
pub struct Query(FileMergedQuery, /* your queries */);

#[derive(Default, MergedObject)]
pub struct Mutation(FileMergedMutation, /* your mutations */);

GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .data(FileConfig {
        bucket: "my-bucket".to_owned(),
        ..Default::default()
    })
    .data(Arc::new(s3_client()))
    .finish()
```

The s3 client is not part of `FileConfig`, build it once with your r2 credentials/endpoint and register it the same way as the db connection. R2 is s3-api-compatible but not aws, so the client needs a custom endpoint, a placeholder region, path-style addressing, and the newer aws checksum headers disabled:

```rs
fn s3_client() -> aws_sdk_s3::Client {
    let config = aws_sdk_s3::Config::builder()
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new("auto"))
        .endpoint_url("https://<account-id>.r2.cloudflarestorage.com")
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            "<access-key-id>",
            "<secret-access-key>",
            None,
            None,
            "r2",
        ))
        .force_path_style(true)
        .build();
    aws_sdk_s3::Client::from_conf(config)
}
```

Your migration must include the `File` table.

## Upload chain

```graphql
mutation {
    fileUploadInit(data: { filename: "walter-notes.pdf", contentType: "application/pdf" }) {
        uploadUrl
        inner { id status }
    }
}
```

The client `PUT`s the raw bytes to `uploadUrl` directly, no server involved, then:

```graphql
mutation { fileUploadConfirm(id: "...") { id status size etag downloadUrl } }
```

`fileUploadConfirm` runs a `HeadObject` against the real key (the client-reported size/type from `fileUploadInit` is never trusted), moves the row to `Uploaded`, and calls `FileHandlers::on_upload_confirm`. The default is a no-op - override it to inspect or transform the file (`ffprobe`, image/video compression, thumbnailing...) and move `status` to `Processing`/`Ready`/`Failed` once that work finishes, outside of the request:

```rs
struct MyHandlers;
#[async_trait]
impl FileHandlers for MyHandlers {
    async fn on_upload_confirm(&self, ctx: &Context<'_>, file: &FileSql) -> Res<()> {
        enqueue_processing_job(file.id.clone()); // ffprobe/compress, then update status yourself
        Ok(())
    }
}

FileConfig {
    bucket: "my-bucket".to_owned(),
    handlers: Arc::new(MyHandlers),
    ..Default::default()
}
```

`downloadUrl` is a presigned get url, computed per request, `None` while `status` is still `Pending`.

`fileDelete` only removes the object from the bucket on a permanent delete (`fileDelete(id: "...", permanent: true)`), a soft delete leaves the object in place so it stays recoverable.

`File.orgId` is a plain nullable column, not a `belongs_to` relation, since this package does not own your `Org` model. Add the reverse edge on your own `Org`:

```rs
#[model]
pub struct Org {
    #[has_many]
    pub files: File,
    // ...
}
```

## Client never confirms

A client can call `fileUploadInit` and then never `PUT` the bytes or never call `fileUploadConfirm` - the row is left `Pending` forever, and if the client did `PUT` without confirming, so is the object. `File.uploadExpiresAt` (`now` + `FileConfig.upload_url_expires_ms`, the same window the presigned put url is valid for) tracks this.

```graphql
mutation { fileCleanupExpiredPending }
```

Finds `Pending` rows whose `uploadExpiresAt` is in the past, best-effort deletes the object (in case one exists despite never being confirmed), and permanently removes the rows, returning the count removed. This package ships no scheduler - call it from your own cron job/worker on whatever interval fits, and gate it with your own auth/authz layer if you don't want it publicly callable, the same way any other custom `#[mutation]` can be wrapped with `authz(realm = "system")`.

## Builtin ffprobe/ffmpeg processing

`FfmpegFileHandlers` is a ready-to-use `FileHandlers` that shells out to the `ffprobe`/`ffmpeg` binaries - both must be installed on the host system, this crate does not vendor, download, or FFI-bind them, it only runs them as a subprocess, gated behind the `ffmpeg` cargo feature (`file_ffmpeg` at the root) so the `tokio` process/fs features are opt-in:

```rs
FileConfig {
    bucket: "my-bucket".to_owned(),
    handlers: Arc::new(FfmpegFileHandlers::default()),
    ..Default::default()
}

// or with custom tuning
FileConfig {
    handlers: Arc::new(FfmpegFileHandlers(FfmpegConfig {
        max_image_dimension: 1600,
        video_crf: 24,
        ..Default::default()
    })),
    ..Default::default()
}
```

On `fileUploadConfirm`, it flips the row to `Processing` (in the same request/transaction), then in a spawned background task: downloads the object, runs `ffprobe -v quiet -print_format json -show_format -show_streams`, and if `content_type` is `image/*`/`video/*`, downscales/recompresses it with `ffmpeg` - if the result is smaller it is re-uploaded over the same key and `size`/`etag` are updated. The background task uses its own raw db connection rather than `ctx.tx()`, since it outlives the request that triggered it - this is also why `on_upload_confirm` only kicks the work off, it does not await it, a huge video transcode must never block a GraphQL response or hold the shared per-request transaction open.

`ffprobe` failing - binary not installed, non-zero exit, unparsable stdout - never fails the file, it is only there to fill in optional display metadata:

- `File.metadata` (`#[graphql(skip)]`) gets the raw `ffprobe` json, `None` if the probe failed.
- `File.mediaWidth` / `mediaHeight` / `mediaCodec` are read from the first stream with `codec_type: "video"` (a single-frame image is reported by `ffprobe` as one video stream too) so a client can read dimensions/codec without fetching and parsing the whole json blob. All three are `None` if the probe failed or found no video stream (e.g. an audio-only upload).

Minifying, on the other hand, is fatal when attempted - if `ffmpeg` cannot make sense of an `image/*`/`video/*` object, the row moves to `Failed` with the error under `metadata.error`, since that is an actual processing failure, not missing-but-optional metadata. Non-media `content_type`s never attempt minify at all, so a failed probe is the only thing that can happen to them, and the row still reaches `Ready`.

`extract_media_info(probe: &JsonValue) -> MediaInfo { width, height, codec }` is exported standalone if you want the same first-video-stream extraction elsewhere.

Write your own `FileHandlers::on_upload_confirm` instead if you need different tooling (a different image library, virus scanning, a real job queue) - `FfmpegFileHandlers`'s `process`/`mark_failed` helpers are a template for that, not a required base to build on.
