use crate::prelude::*;

/// Width/height/codec of the first video stream found in an ffprobe -show_streams json
/// output, a single-frame image is reported by ffprobe as one video stream too.
#[derive(Default)]
pub struct MediaInfo {
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub codec: Option<String>,
}

/// Reads probe (the raw json from ffprobe) for the first stream with codec_type "video",
/// missing/malformed fields are left None rather than erroring, this is metadata extraction
/// for display purposes, not something that should fail the upload it is attached to.
pub fn extract_media_info(probe: &JsonValue) -> MediaInfo {
    let Some(stream) = probe
        .get("streams")
        .and_then(JsonValue::as_array)
        .and_then(|streams| streams.iter().find(|s| s.get("codec_type").and_then(JsonValue::as_str) == Some("video")))
    else {
        return MediaInfo::default();
    };

    MediaInfo {
        width: stream.get("width").and_then(JsonValue::as_i64),
        height: stream.get("height").and_then(JsonValue::as_i64),
        codec: stream.get("codec_name").and_then(JsonValue::as_str).map(ToOwned::to_owned),
    }
}
