use super::prelude::*;

/// Access to the raw connection pool stored in GrandLineData.
#[async_trait]
pub trait DbContext<'a>
where
    Self: GrandLineDataContext<'a>,
{
    /// The pool itself, outside the request's own transaction. For a write that
    /// must survive a rollback, e.g. an otp attempt counter. Everything else wants
    /// ctx.db(), which respects whether this request is transactional.
    async fn db_pool(&self) -> Res<&'a DatabaseConnection> {
        let db = self.grand_line()?.db_pool.as_ref();
        Ok(db)
    }

    /// The connection for this resolver: the request transaction when the request
    /// writes, a pooled connection when it only reads. Prefer this everywhere, it
    /// is what the crud macros inject as db.
    async fn db(&self) -> Res<ConnX<'a>> {
        self.grand_line()?.db().await
    }

    /// Forces the request transaction open and returns it, for a read whose
    /// operation type says query but which is about to write anyway. Every later
    /// db() returns the transaction too, so nothing reads around its own write.
    async fn tx(&self) -> Res<ConnX<'a>> {
        self.grand_line()?.tx().await
    }

    /// Commits the request transaction now and hands its connection back to the
    /// pool, for a resolver whose response outlives the request. The #[subscribe]
    /// macro calls this once its guards have run, so a live subscription holds no
    /// transaction and later reads come from the pool.
    async fn tx_finish(&self) -> Res<()> {
        self.grand_line()?.tx_finish().await
    }

    /// Queues work to run after this request commits, for a mutation that kicks off
    /// something slow (a subprocess, a transcode) without holding the request
    /// transaction and its connection open for the duration.
    ///
    /// The job is spawned only on a successful commit, a rollback drops it: there is
    /// no background work to do for a request that did not land. It receives a
    /// pooled connection, so it cannot capture or write through the request
    /// transaction and race the commit.
    async fn detach<F, Fu>(&self, f: F) -> Res<()>
    where
        F: FnOnce(Arc<DatabaseConnection>) -> Fu + Send + 'static,
        Fu: Future<Output = Res<()>> + Send + 'static,
    {
        self.grand_line()?.detach(f).await;
        Ok(())
    }
}

#[async_trait]
impl<'a> DbContext<'a> for Context<'a> {
}
