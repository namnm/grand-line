use crate::prelude::*;

/// Implemented by every core error enum to describe how it is exposed to graphql clients.
pub trait GrandLineErrImpl
where
    Self: Error + Send + Sync,
{
    /// Machine-readable error code, exposed to the client only when client() is true.
    fn code(&self) -> &'static str;
    /// Whether this error is safe to expose to the client, if false it is masked as internal server error.
    fn client(&self) -> bool;
    /// Builds the graphql error extensions map, falling back to the internal server error code when client() is false.
    fn extensions(&self) -> ErrorExtensionValues {
        let mut m = ErrorExtensionValues::default();
        m.set(
            "code",
            if self.client() {
                self.code()
            } else {
                CoreGraphQLErr::InternalServer.code()
            },
        );
        m
    }
}
