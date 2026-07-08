#[path = "./setup.rs"]
mod setup;
use setup::*;

#[test]
fn extract_media_info_reads_the_first_video_stream() {
    let probe = json!({
        "streams": [
            { "codec_type": "audio", "codec_name": "aac" },
            { "codec_type": "video", "codec_name": "h264", "width": 1920, "height": 1080 },
        ],
    });

    let media = extract_media_info(&probe);

    pretty_eq!(media.width, Some(1920), "width should come from the first video stream");
    pretty_eq!(media.height, Some(1080), "height should come from the first video stream");
    pretty_eq!(media.codec, Some("h264".to_owned()), "codec should come from the first video stream");
}

#[test]
fn extract_media_info_is_empty_when_there_is_no_video_stream() {
    let probe = json!({
        "streams": [
            { "codec_type": "audio", "codec_name": "aac" },
        ],
    });

    let media = extract_media_info(&probe);

    pretty_eq!(media.width, None, "width should be None for an audio only probe");
    pretty_eq!(media.height, None, "height should be None for an audio only probe");
    pretty_eq!(media.codec, None, "codec should be None for an audio only probe");
}

#[test]
fn extract_media_info_is_empty_for_a_malformed_probe() {
    let probe = json!({ "olivia": "dunham" });

    let media = extract_media_info(&probe);

    pretty_eq!(media.width, None, "width should be None when streams is missing");
    pretty_eq!(media.height, None, "height should be None when streams is missing");
    pretty_eq!(media.codec, None, "codec should be None when streams is missing");
}
