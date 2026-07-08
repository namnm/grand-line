use crate::prelude::*;

/// Access to the s3/r2 client registered on the schema, see the readme for how to build one.
pub trait FileS3Context<'a>
where
    Self: ImplContext<'a>,
{
    /// Returns the s3 client registered on the context via .data(Arc::new(client)),
    /// err MyErr::S3ClientMissing if it was not registered.
    fn file_s3_client(&self) -> Res<&'a Arc<aws_sdk_s3::Client>> {
        self.data_opt_impl::<Arc<aws_sdk_s3::Client>>().ok_or(MyErr::S3ClientMissing.into())
    }
}

impl<'a> FileS3Context<'a> for Context<'a> {
}
