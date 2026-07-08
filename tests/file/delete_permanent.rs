#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn delete_permanent_removes_the_object_and_the_row() -> Res<()> {
    let deleted = mock_delete_object_response()?;
    let req = ::http::Request::builder()
        .uri("http://mock/astrid/photo.jpg")
        .body(aws_smithy_types::body::SdkBody::empty())
        .map_err(err_s3)?;
    let d = setup(vec![ReplayEvent::new(req, deleted)]).await?;

    let f = am_create!(File {
        key: "astrid/photo.jpg",
        filename: "photo.jpg",
        content_type: "image/jpeg",
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    let v = value!({
        "id": f.id.clone(),
        "permanent": true,
    });
    exec_assert_ok(&d.s, Q_DELETE, Some(v)).await;

    let r = File::find_by_id(&f.id).include_deleted(true).one(&d.tmp.db).await?;
    pretty_eq!(r.is_none(), true, "row should be permanently removed from the db");

    d.tmp.drop().await
}
