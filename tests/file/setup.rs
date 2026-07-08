#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

pub use grand_line::prelude::*;

// ---------------------------------------------------------------------------
// File GraphQL query fixtures
// ---------------------------------------------------------------------------

pub const Q_UPLOAD_INIT: &str = "
mutation test($data: FileUploadInit!) {
    fileUploadInit(data: $data) {
        uploadUrl
        inner {
            id
            key
            status
            filename
            contentType
        }
    }
}
";

pub const Q_UPLOAD_CONFIRM: &str = "
mutation test($id: String!) {
    fileUploadConfirm(id: $id) {
        id
        status
        size
        etag
        contentType
        downloadUrl
    }
}
";

pub const Q_DELETE: &str = "
mutation test($id: String!, $permanent: Boolean) {
    fileDelete(id: $id, permanent: $permanent) {
        id
    }
}
";

pub const Q_DETAIL: &str = "
query test($id: ID!) {
    fileDetail(id: $id) {
        id
        key
        status
    }
}
";

pub const Q_CLEANUP_EXPIRED_PENDING: &str = "
mutation test {
    fileCleanupExpiredPending
}
";

// ---------------------------------------------------------------------------
// Query and mutation root types
// ---------------------------------------------------------------------------

#[derive(Default, MergedObject)]
pub struct Query(FileMergedQuery);
#[derive(Default, MergedObject)]
pub struct Mutation(FileMergedMutation);

// ---------------------------------------------------------------------------
// Test fixture setup
// ---------------------------------------------------------------------------

pub const BUCKET: &str = "walter-labs";

pub struct Setup {
    pub tmp: TmpDb,
    pub s: GraphQLSchema<Query, Mutation, EmptySubscription>,
}

/// Builds a schema wired to a temporary db and a mock s3 client that replays events in order.
pub async fn setup(events: Vec<ReplayEvent>) -> Res<Setup> {
    let tmp = tmp_db!(File);

    let c = FileConfig {
        bucket: BUCKET.to_owned(),
        handlers: Arc::new(MockFileHandlers),
        ..Default::default()
    };
    let client = mock_s3_client(events);

    let s = schema_qm::<Query, Mutation>(&tmp.db).data(c).data(Arc::new(client)).finish();

    Ok(Setup {
        tmp,
        s,
    })
}

// ---------------------------------------------------------------------------
// Mock file handlers
// ---------------------------------------------------------------------------

pub struct MockFileHandlers;
#[async_trait]
impl FileHandlers for MockFileHandlers {
    async fn key(&self, _ctx: &Context<'_>, filename: &str) -> Res<String> {
        Ok(format!("olivia/{filename}"))
    }

    async fn on_upload_confirm(&self, ctx: &Context<'_>, file: &FileSql) -> Res<()> {
        let tx = &*ctx.tx().await?;
        am_update!(File {
            id: file.id.clone(),
            metadata: Some(json!({ "processed_by": "mock_file_handlers" })),
        })
        .exec_without_ctx(tx)
        .await?;
        Ok(())
    }
}
