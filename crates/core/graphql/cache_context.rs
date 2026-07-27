use super::prelude::*;

#[async_trait]
pub trait CacheContext<'a>
where
    Self: GrandLineDataContext<'a>,
{
    /// Returns the cached value of type T, running init to populate it on first use.
    async fn cache<T, F, Fu>(&self, init: F) -> Res<Arc<T>>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> Fu + Send,
        Fu: Future<Output = Res<T>> + Send,
    {
        // Clone the per-type OnceCell out and release the whole-map lock before
        // awaiting init, so populating one type's cache does not block every other
        // type's cache() call for the rest of the request, and so init() is free to
        // call ctx.cache::<OtherType>() itself without deadlocking on this lock.
        let cell = {
            let mut m = self.grand_line()?.cache.lock().await;
            Arc::clone(m.entry(TypeId::of::<T>()).or_insert_with(|| Arc::new(OnceCell::new())))
        };

        let arc = cell
            .get_or_try_init(async move || {
                let arc = Arc::new(init().await?);
                Ok::<_, GrandLineErr>(arc as ArcAny)
            })
            .await?;

        let v = Arc::clone(arc).downcast::<T>().map_err(|_| MyErr::CacheDowncast)?;

        Ok(v)
    }

    /// Returns the cached value of type T if it was already populated, None otherwise.
    async fn get_cache<T>(&self) -> Res<Option<Arc<T>>>
    where
        T: Send + Sync + 'static,
    {
        let mut m = self.grand_line()?.cache.lock().await;

        let cell = m.entry(TypeId::of::<T>()).or_insert_with(|| Arc::new(OnceCell::new()));
        let Some(arc) = cell.get() else {
            return Ok(None);
        };

        let v = Arc::clone(arc).downcast::<T>().map_err(|_| MyErr::CacheDowncast)?;
        drop(m);

        Ok(Some(v))
    }
}

#[async_trait]
impl<'a> CacheContext<'a> for Context<'a> {
}
