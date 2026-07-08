use crate::prelude::*;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{BehaviorVersion, Config, Credentials, Region};
use aws_smithy_runtime::client::http::test_util::StaticReplayClient;
use aws_smithy_types::body::SdkBody;

pub use aws_smithy_runtime::client::http::test_util::ReplayEvent;

/// Builds an s3 Client backed by a StaticReplayClient replaying events in order, for tests
/// that exercise fileUploadConfirm/permanent delete without a real bucket.
pub fn mock_s3_client(events: Vec<ReplayEvent>) -> Client {
    let http_client = StaticReplayClient::new(events);
    let config = Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("auto"))
        .credentials_provider(Credentials::new("test", "test", None, None, "test"))
        .force_path_style(true)
        .http_client(http_client)
        .build();
    Client::from_conf(config)
}

/// Canned 200 HeadObject response with content_length/content_type/etag headers set.
pub fn mock_head_object_response(size: i64, content_type: &str, etag: &str) -> Res<::http::Response<SdkBody>> {
    let r = ::http::Response::builder()
        .status(200)
        .header("content-length", size.to_string())
        .header("content-type", content_type)
        .header("etag", format!("\"{etag}\""))
        .body(SdkBody::empty())
        .map_err(err_s3)?;
    Ok(r)
}

/// Canned 204 DeleteObject response.
pub fn mock_delete_object_response() -> Res<::http::Response<SdkBody>> {
    let r = ::http::Response::builder().status(204).body(SdkBody::empty()).map_err(err_s3)?;
    Ok(r)
}
