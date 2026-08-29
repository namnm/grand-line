use super::prelude::*;
use async_graphql::futures_util::stream::{self, BoxStream, StreamExt as _};
use async_graphql::parser::types::{DocumentOperations, ExecutableDocument, OperationType};

/// Extension to insert GrandLineData on each request, then cleanup at the end of each request.
/// The extension also handle error automatically to only expose client errors to the client.
pub struct GrandLineExtension;

impl ExtensionFactory for GrandLineExtension {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(GrandLineExtensionImpl)
    }
}

struct GrandLineExtensionImpl;

/// Releases the request transaction when dropped, see GrandLineExtension::subscribe.
struct TxRelease(Arc<GrandLineData>);

impl Drop for TxRelease {
    fn drop(&mut self) {
        self.0.tx_release();
    }
}

#[async_trait]
impl Extension for GrandLineExtensionImpl {
    /// Insert GrandLineData on each request.
    async fn prepare_request(
        &self,
        ctx: &ExtensionContext<'_>,
        request: Request,
        next: NextPrepareRequest<'_>,
    ) -> ServerResult<Request> {
        let db = ctx.data_opt::<Arc<DatabaseConnection>>().ok_or(MyErr::CtxDb404)?;
        let gl = GrandLineData::new(Arc::clone(db));
        // Record the operation the client selected before the document is
        // parsed, parse_query then classifies only that operation.
        gl.set_operation_name(request.operation_name.clone()).await;
        next.run(ctx, request.data(Arc::new(gl))).await
    }

    /// Decide whether this request writes, before any resolver runs. Only the
    /// operation the client selected decides this: a document carrying both a
    /// query and an unused mutation must not pin a transaction for the query,
    /// which is what resolvers.md#connections-and-transactions promises. Only a
    /// mutation needs a transaction, so a query and a subscription read from the
    /// pool instead, paying for no BEGIN and pinning no connection.
    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let doc = next.run(ctx, query, variables).await?;
        let gl = ctx.grand_line().ok();
        let op_name = match gl.as_ref() {
            Some(gl) => gl.operation_name().await,
            None => None,
        };
        let write = match op_name.as_deref() {
            Some(n) => match &doc.operations {
                // Single is always the one anonymous operation, a named
                // operationName never selects it, async-graphql rejects the
                // request right after.
                DocumentOperations::Single(_) => false,
                DocumentOperations::Multiple(m) => m.get(n).is_some_and(|o| o.node.ty == OperationType::Mutation),
            },
            // No operationName: an ambiguous multi-operation document is
            // rejected by async-graphql later anyway, so keep the conservative
            // any-operation-may-write behavior.
            None => doc.operations.iter().any(|(_, o)| o.node.ty == OperationType::Mutation),
        };
        if write && let Some(gl) = gl {
            gl.set_write();
        }
        Ok(doc)
    }

    /// Release the request transaction when a subscription stream ends or is
    /// dropped. execute() never runs for a subscription, so the cleanup that
    /// normally happens there has to hang off the stream's own lifetime instead.
    fn subscribe<'s>(
        &self,
        ctx: &ExtensionContext<'_>,
        stream: BoxStream<'s, Response>,
        next: NextSubscribe<'_>,
    ) -> BoxStream<'s, Response> {
        let stream = next.run(ctx, stream);
        let Some(gl) = ctx.data_opt_impl::<Arc<GrandLineData>>().map(Arc::clone) else {
            return stream;
        };
        // Held by the stream state, so a client disconnecting mid stream releases
        // the transaction just like a stream that ends on its own.
        stream::unfold((stream, TxRelease(gl)), |(mut s, guard)| async move {
            let r = s.next().await?;
            Some((r, (s, guard)))
        })
        .boxed()
    }

    /// Cleanup GrandLineData at the end of each request.
    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        let mut r = next.run(ctx, operation_name).await;
        match ctx.grand_line() {
            Ok(gl) => match gl.cleanup(!r.errors.is_empty()).await {
                Ok(c) => {
                    if c.rolled_back {
                        // data still holds whatever the resolvers produced before the
                        // error, and none of it exists any more. A client reading only
                        // data would see rows that were never written. Only a request
                        // that actually had a transaction gets nulled, so a query
                        // keeps graphql's partial success, it undid nothing.
                        r.data = GraphQLValue::Null;
                    }
                    let broker = ctx.subscription_config().broker();
                    for e in c.events {
                        if let Err(e) = broker.publish(e).await {
                            r.errors.push(e.into());
                        }
                    }
                }
                Err(e) => {
                    // Nothing was persisted, so the data collected from the resolvers
                    // never made it to the database. Returning it would read as a
                    // success to any client that only looks at data.
                    r.data = GraphQLValue::Null;
                    r.errors.push(e.into());
                }
            },
            Err(e) => {
                r.data = GraphQLValue::Null;
                r.errors.push(e.into());
            }
        }
        for e in &mut r.errors {
            // source is None for errors that never reached a resolver, e.g. GraphQL
            // parse errors, unknown field, or a variable type mismatch. Those are
            // reports about the client's own malformed request, not internal server
            // detail, so passing the message through as-is is correct, not a masking
            // gap: no GrandLineErr can ever attach a source to these, they are raised
            // by async-graphql itself before any resolver body runs.
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
                let msg = &e.message;
                eprintln!("{err_path} {msg}");
                e.message = MyErr::InternalServer.to_string();
                e.source = None;
                e.extensions = Some(MyErr::InternalServer.extensions());
            }
        }
        r
    }
}
