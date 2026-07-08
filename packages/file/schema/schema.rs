use crate::prelude::*;

/// Combined GraphQL query root for the file package, merge into the host schema.
#[derive(Default, MergedObject)]
pub struct FileMergedQuery(FileSearchQuery, FileCountQuery, FileDetailQuery);

/// Combined GraphQL mutation root for the file package, merge into the host schema.
#[derive(Default, MergedObject)]
pub struct FileMergedMutation(
    FileUploadInitMutation,
    FileUploadConfirmMutation,
    FileDeleteMutation,
    FileCleanupExpiredPendingMutation,
);
