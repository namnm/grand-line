use crate::prelude::*;
use aws_sdk_s3::primitives::ByteStream;
use std::path::{Path, PathBuf};
use tokio::process::Command;

// ---------------------------------------------------------------------------
// Temp file with best-effort cleanup on drop
// ---------------------------------------------------------------------------

/// A path under the system temp dir, removed on drop, ffmpeg/ffprobe need a real file on
/// disk, s3 objects are downloaded/uploaded through one of these rather than kept in memory.
pub struct TempPath(PathBuf);

impl TempPath {
    /// Builds a fresh, not yet created, temp path with the given extension.
    pub fn new(ext: &str) -> Self {
        let name = format!("grand_line_file_{}.{ext}", ulid());
        Self(std::env::temp_dir().join(name))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Extension to use for a temp file representing filename, falls back to "bin".
pub fn ext_of(filename: &str) -> &str {
    Path::new(filename).extension().and_then(|e| e.to_str()).unwrap_or("bin")
}

// ---------------------------------------------------------------------------
// s3 <-> local file transfer
// ---------------------------------------------------------------------------

/// Downloads bucket/key into path.
pub async fn download_to(client: &aws_sdk_s3::Client, bucket: &str, key: &str, path: &Path) -> Res<()> {
    let out = client.get_object().bucket(bucket).key(key).send().await.map_err(err_s3)?;
    let bytes = out.body.collect().await.map_err(err_s3)?.into_bytes();
    tokio::fs::write(path, &bytes).await.map_err(err_s3)?;
    Ok(())
}

/// Uploads path to bucket/key, returns the new etag if the bucket reports one.
pub async fn upload_from(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    content_type: &str,
    path: &Path,
) -> Res<Option<String>> {
    let bytes = tokio::fs::read(path).await.map_err(err_s3)?;
    let out = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type(content_type)
        .body(ByteStream::from(bytes))
        .send()
        .await
        .map_err(err_s3)?;
    Ok(out.e_tag().map(ToOwned::to_owned))
}

// ---------------------------------------------------------------------------
// ffprobe / ffmpeg subprocess helpers
// ---------------------------------------------------------------------------

/// Runs program with args, err MyErr::Process on a non 0 exit code or a spawn failure
/// (e.g. the binary is not installed), stdout is returned on success.
async fn run(program: &str, args: &[String]) -> Res<Vec<u8>> {
    let out = Command::new(program).args(args).output().await.map_err(|e| MyErr::Process {
        program: program.to_owned(),
        inner: e.to_string(),
    })?;
    if !out.status.success() {
        return Err(MyErr::Process {
            program: program.to_owned(),
            inner: String::from_utf8_lossy(&out.stderr).into_owned(),
        }
        .into());
    }
    Ok(out.stdout)
}

/// Runs ffprobe on path, returns the parsed -show_format -show_streams json output.
pub async fn ffprobe(bin: &str, path: &Path) -> Res<JsonValue> {
    let args = vec![
        "-v".to_owned(),
        "quiet".to_owned(),
        "-print_format".to_owned(),
        "json".to_owned(),
        "-show_format".to_owned(),
        "-show_streams".to_owned(),
        path.to_string_lossy().into_owned(),
    ];
    let out = run(bin, &args).await?;
    let v = serde_json::from_slice(&out)?;
    Ok(v)
}

/// Downscales/recompresses an image with ffmpeg, output keeps the input's aspect ratio,
/// bounded by max_dimension on its longest side, quality is 1 (worst) to 100 (best).
pub async fn minify_image(bin: &str, input: &Path, output: &Path, max_dimension: u32, quality: u8) -> Res<()> {
    let qv = 2 + (100 - u32::from(quality.clamp(1, 100))) * 29 / 100;
    let scale = format!("scale='min(iw,{max_dimension})':'min(ih,{max_dimension})':force_original_aspect_ratio=decrease");
    let args = vec![
        "-y".to_owned(),
        "-i".to_owned(),
        input.to_string_lossy().into_owned(),
        "-vf".to_owned(),
        scale,
        "-q:v".to_owned(),
        qv.to_string(),
        output.to_string_lossy().into_owned(),
    ];
    run(bin, &args).await?;
    Ok(())
}

/// Downscales/recompresses a video with ffmpeg, output keeps the input's aspect ratio,
/// bounded by max_height, crf is 0 (lossless/largest) to 51 (smallest/worst).
pub async fn minify_video(bin: &str, input: &Path, output: &Path, max_height: u32, crf: u8) -> Res<()> {
    let scale = format!("scale=-2:'min(ih,{max_height})'");
    let args = vec![
        "-y".to_owned(),
        "-i".to_owned(),
        input.to_string_lossy().into_owned(),
        "-vf".to_owned(),
        scale,
        "-c:v".to_owned(),
        "libx264".to_owned(),
        "-crf".to_owned(),
        crf.to_string(),
        "-preset".to_owned(),
        "veryfast".to_owned(),
        "-c:a".to_owned(),
        "aac".to_owned(),
        "-b:a".to_owned(),
        "128k".to_owned(),
        output.to_string_lossy().into_owned(),
    ];
    run(bin, &args).await?;
    Ok(())
}
