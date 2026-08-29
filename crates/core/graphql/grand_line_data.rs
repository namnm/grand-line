use super::prelude::*;

/// What cleanup did, so the caller knows whether the response still describes
/// anything that exists.
pub struct Cleanup {
    /// Subscription events to publish, empty on a rollback.
    pub events: Vec<SubscriptionEvent>,
    /// True when a transaction was actually rolled back, so everything the
    /// resolvers produced is gone. False for a read, which had nothing to undo.
    pub rolled_back: bool,
}

/// GrandLineData should be constructed on each request.
/// We will get it in the resolvers to manage per-request db transaction, graphql loaders, cache...
/// We should only use it in the GrandLineExtension to inject this context automatically on each request.
pub struct GrandLineData {
    pub(crate) db_pool: Arc<DatabaseConnection>,
    /// Owned, not an Arc: committing consumes the transaction, so handing out
    /// clones would let anything holding one block the commit. Resolvers borrow it
    /// through ConnX instead, see db().
    pub(crate) tx: Mutex<Option<DatabaseTransaction>>,
    /// Whether this request writes, decided from the operation type before any
    /// resolver runs, see GrandLineExtension::parse_query. A read gets a pooled
    /// connection and never pays for a transaction it does not need.
    pub(crate) write: AtomicBool,
    pub(crate) loaders: Mutex<HashMap<String, ArcAny>>,
    pub(crate) cache: Mutex<HashMap<TypeId, Arc<OnceCell<ArcAny>>>>,
    pub(crate) events: Mutex<Vec<SubscriptionEvent>>,
    /// Work queued to run after this request commits, see detach.
    pub(crate) detached: Mutex<Vec<BoxFuture<'static, ()>>>,
    /// The operationName the client selected, recorded from the request before
    /// the document is parsed, see GrandLineExtension::parse_query.
    pub(crate) operation_name: Mutex<Option<String>>,
}

impl GrandLineData {
    pub(crate) fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db_pool: db,
            tx: Mutex::new(None),
            write: AtomicBool::new(false),
            loaders: Mutex::new(HashMap::new()),
            cache: Mutex::new(HashMap::new()),
            events: Mutex::new(vec![]),
            detached: Mutex::new(vec![]),
            operation_name: Mutex::new(None),
        }
    }

    /// Records the operation the client selected, before the document is parsed.
    pub(crate) async fn set_operation_name(&self, n: Option<String>) {
        *self.operation_name.lock().await = n;
    }

    /// The operation the client selected, if it named one.
    pub(crate) async fn operation_name(&self) -> Option<String> {
        self.operation_name.lock().await.clone()
    }

    /// Marks this request as writing, so db() hands out the transaction.
    pub(crate) fn set_write(&self) {
        self.write.store(true, Ordering::SeqCst);
    }

    /// The connection for this request: the transaction when it writes or when one
    /// was already opened explicitly, a pooled connection otherwise.
    pub(crate) async fn db(&self) -> Res<ConnX<'_>> {
        if !self.write.load(Ordering::SeqCst) && self.tx.lock().await.is_none() {
            return Ok(ConnX::db(&self.db_pool));
        }
        self.tx_begin().await?;
        Ok(ConnX::tx(&self.db_pool, &self.tx))
    }

    /// The request transaction, opening it if this request had not needed one yet.
    /// Every later db() then returns it too, so a read never sees less than the
    /// write that preceded it.
    pub(crate) async fn tx(&self) -> Res<ConnX<'_>> {
        self.set_write();
        self.tx_begin().await?;
        Ok(ConnX::tx(&self.db_pool, &self.tx))
    }

    async fn tx_begin(&self) -> Res<()> {
        let mut guard = self.tx.lock().await;
        if guard.is_none() {
            *guard = Some(self.db_pool.begin().await?);
        }
        drop(guard);
        Ok(())
    }

    /// Queues f to run after this request commits, see DbContext::detach.
    /// f is handed a pooled connection rather than the request transaction, so a
    /// job that outlives the request cannot write through a transaction the
    /// request owns and is about to consume.
    pub(crate) async fn detach<F, Fu>(&self, f: F)
    where
        F: FnOnce(Arc<DatabaseConnection>) -> Fu + Send + 'static,
        Fu: Future<Output = Res<()>> + Send + 'static,
    {
        let db = Arc::clone(&self.db_pool);
        let fu = f(db);
        let job = Box::pin(async move {
            if let Err(e) = fu.await {
                eprintln!("detached job failed: {e}");
            }
        });
        self.detached.lock().await.push(job);
    }

    /// Ends the request and returns the subscription events to publish, empty on a
    /// rollback: a change nobody committed is a change nobody should hear about.
    /// Detached jobs follow the same rule, and a failing commit drops them with the
    /// request rather than running work for writes that never landed.
    pub(crate) async fn cleanup(&self, error: bool) -> Res<Cleanup> {
        self.loaders.lock().await.clear();
        let events = self.events.lock().await.drain(..).collect::<Vec<_>>();
        if error {
            let rolled_back = self.rollback().await?;
            self.detached.lock().await.clear();
            return Ok(Cleanup {
                events: vec![],
                rolled_back,
            });
        }
        self.commit().await?;
        self.detached_spawn().await;
        Ok(Cleanup {
            events,
            rolled_back: false,
        })
    }

    /// Spawns every queued detached job, only ever called after a successful commit.
    async fn detached_spawn(&self) {
        let jobs = self.detached.lock().await.drain(..).collect::<Vec<_>>();
        for j in jobs {
            spawn(j);
        }
    }

    /// Commits the transaction now and releases its connection back to the pool,
    /// for a resolver that is done with the database but whose response outlives
    /// the request, i.e. a subscription. A later db() reads from the pool again.
    pub(crate) async fn tx_finish(&self) -> Res<()> {
        self.commit().await?;
        self.write.store(false, Ordering::SeqCst);
        // cleanup never runs for a subscription: execute() is skipped and the
        // stream's lifetime is handled by TxRelease instead. Without this, a
        // resolver detaching work would queue a job that never runs, breaking
        // detach's promise to run it after the request commits. Same rule as
        // cleanup: only ever after a successful commit, a failed commit drops
        // the queue with the request rather than running work for writes that
        // never landed.
        self.detached_spawn().await;
        Ok(())
    }

    /// Drops the transaction without awaiting anything, sea_orm rolls it back on
    /// drop and hands the connection back. The escape hatch for a request that
    /// never reaches cleanup, see GrandLineExtension::subscribe.
    pub(crate) fn tx_release(&self) {
        if let Ok(mut guard) = self.tx.try_lock() {
            drop(guard.take());
        }
    }

    async fn commit(&self) -> Res<()> {
        let tx = self.tx.lock().await.take();
        if let Some(tx) = tx {
            tx.commit().await?;
        }
        Ok(())
    }

    /// Rolls the request transaction back, returning whether there was one.
    /// A read never opens one, so nothing it produced was undone.
    async fn rollback(&self) -> Res<bool> {
        let tx = self.tx.lock().await.take();
        let Some(tx) = tx else {
            return Ok(false);
        };
        tx.rollback().await?;
        Ok(true)
    }
}
