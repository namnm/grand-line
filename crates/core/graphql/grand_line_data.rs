use super::prelude::*;

/// GrandLineData should be constructed on each request.
/// We will get it in the resolvers to manage per-request db transaction, graphql loaders, cache...
/// We should only use it in the GrandLineExtension to inject this context automatically on each request.
pub struct GrandLineData {
    pub(crate) db: Arc<DatabaseConnection>,
    pub(crate) tx: Mutex<Option<Arc<DatabaseTransaction>>>,
    pub(crate) loaders: Mutex<HashMap<String, ArcAny>>,
    pub(crate) cache: Mutex<HashMap<TypeId, Arc<OnceCell<ArcAny>>>>,
}

impl GrandLineData {
    pub(crate) fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db,
            tx: Mutex::new(None),
            loaders: Mutex::new(HashMap::new()),
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) async fn tx(&self) -> Res<Arc<DatabaseTransaction>> {
        let mut guard = self.tx.lock().await;
        if let Some(a) = &*guard {
            let a = Arc::clone(a);
            drop(guard);
            Ok(a)
        } else {
            let tx = Arc::new(self.db.begin().await?);
            *guard = Some(Arc::clone(&tx));
            drop(guard);
            Ok(tx)
        }
    }

    pub(crate) async fn cleanup(&self, error: bool) -> Res<()> {
        self.loaders.lock().await.clear();
        if error {
            self.rollback().await
        } else {
            self.commit().await
        }
    }

    async fn commit(&self) -> Res<()> {
        let tx = self.tx.lock().await.take();
        if let Some(tx) = tx {
            match Arc::try_unwrap(tx) {
                Ok(tx) => {
                    tx.commit().await?;
                }
                Err(_) => {
                    return Err(MyErr::TxCommit.into());
                }
            }
        }
        Ok(())
    }

    async fn rollback(&self) -> Res<()> {
        let tx = self.tx.lock().await.take();
        if let Some(tx) = tx {
            match Arc::try_unwrap(tx) {
                Ok(tx) => {
                    tx.rollback().await?;
                }
                Err(_) => {
                    return Err(MyErr::TxRollback.into());
                }
            }
        }
        Ok(())
    }
}
