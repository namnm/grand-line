use super::prelude::*;

/// Extension to insert `GrandLineContextData` on each request, then cleanup at the end of each request.
/// The extension also handle error automatically to only expose client errors to the client.
pub struct GrandLineExtension;

impl ExtensionFactory for GrandLineExtension {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(GrandLineExtensionImpl)
    }
}

struct GrandLineExtensionImpl;

#[async_trait]
impl Extension for GrandLineExtensionImpl {
    /// Insert `GrandLineContextData` on each request.
    async fn prepare_request(
        &self,
        ctx: &ExtensionContext<'_>,
        request: Request,
        next: NextPrepareRequest<'_>,
    ) -> ServerResult<Request> {
        let db = ctx.data_opt::<Arc<DatabaseConnection>>().ok_or(MyErr::CtxDb404)?;
        let gl = GrandLineContextData::new(Arc::clone(db));
        next.run(ctx, request.data(Arc::new(gl))).await
    }

    /// Cleanup `GrandLineContextData` at the end of each request.
    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        let mut r = next.run(ctx, operation_name).await;
        match ctx.grand_line() {
            Ok(gl) => {
                if let Err(e) = gl.cleanup(!r.errors.is_empty()).await {
                    r.errors.push(e.into());
                }
            }
            Err(e) => {
                r.errors.push(e.into());
            }
        }
        for e in &mut r.errors {
            if e.source.is_none() {
                continue;
            }
            let gl = e.source.as_deref().and_then(|e| e.downcast_ref::<GrandLineErr>());
            if let Some(GrandLineErr(gl)) = gl
                && gl.client()
            {
                e.extensions = Some(gl.extensions());
            } else {
                let mut err_path = e
                    .path
                    .iter()
                    .map(|s| match s {
                        PathSegment::Field(f) => f.to_owned(),
                        PathSegment::Index(i) => i.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(".");
                if err_path.is_empty() {
                    err_path = "<unknown>".to_owned();
                }
                eprintln!("{} {}", err_path, e.message);
                e.message = MyErr::InternalServer.to_string();
                e.source = None;
                e.extensions = Some(MyErr::InternalServer.extensions());
            }
        }
        r
    }
}
