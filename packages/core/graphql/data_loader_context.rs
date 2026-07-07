use super::prelude::*;
use dataloader::DataLoader;
use tokio::spawn;

#[async_trait]
pub trait DataLoaderContext<'a>
where
    Self: GrandLineDataContext<'a>,
{
    /// Returns the cached DataLoader for key, creating and caching a new one on first use.
    async fn data_loader<E>(
        &self,
        key: String,
        col: E::C,
        look_ahead: Vec<LookaheadX<E>>,
        condition: Condition,
    ) -> Res<Arc<DataLoader<LoaderX<E>>>>
    where
        E: EntityX,
    {
        let gl = self.grand_line()?;
        let mut guard = gl.loaders.lock().await;
        let a = if let Some(a) = guard.get(&key) {
            let a = Arc::clone(a);
            drop(guard);
            a.downcast::<DataLoader<LoaderX<E>>>()
                .map_err(|_| MyErr::LoaderDowncast)?
        } else {
            let a = Arc::new(DataLoader::new(
                LoaderX {
                    tx: gl.tx().await?,
                    col,
                    look_ahead,
                    condition,
                },
                spawn,
            ));
            guard.insert(key, Arc::<DataLoader<LoaderX<E>>>::clone(&a));
            drop(guard);
            a
        };
        Ok(a)
    }
}

#[async_trait]
impl<'a> DataLoaderContext<'a> for Context<'a> {
}
