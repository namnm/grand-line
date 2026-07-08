use crate::prelude::*;

/// Tuning knobs for FfmpegFileHandlers, see the readme for the ffprobe/ffmpeg requirement.
#[derive(Clone)]
pub struct FfmpegConfig {
    /// Binary name or path used to run ffprobe, "ffprobe" resolved from PATH by default.
    pub ffprobe_bin: String,
    /// Binary name or path used to run ffmpeg, "ffmpeg" resolved from PATH by default.
    pub ffmpeg_bin: String,
    /// Whether to downscale/recompress files whose content_type starts with "image/".
    pub minify_images: bool,
    /// Whether to downscale/recompress files whose content_type starts with "video/".
    pub minify_videos: bool,
    /// Images wider or taller than this are downscaled, aspect ratio preserved.
    pub max_image_dimension: u32,
    /// 1 (smallest/worst) to 100 (largest/best), mapped to ffmpeg's mjpeg -q:v scale.
    pub image_quality: u8,
    /// Videos taller than this are downscaled, aspect ratio preserved.
    pub max_video_height: u32,
    /// x264 constant rate factor, 0 (lossless/largest) to 51 (smallest/worst).
    pub video_crf: u8,
}

impl Default for FfmpegConfig {
    fn default() -> Self {
        Self {
            ffprobe_bin: "ffprobe".to_owned(),
            ffmpeg_bin: "ffmpeg".to_owned(),
            minify_images: true,
            minify_videos: true,
            max_image_dimension: 2048,
            image_quality: 80,
            max_video_height: 720,
            video_crf: 28,
        }
    }
}
