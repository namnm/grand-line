use super::prelude::*;
use dataloader::Loader;

/// Batch loader for entity E, keyed by the string value of col.
pub struct LoaderX<E>
where
    E: EntityX,
{
    pub tx: Arc<DatabaseTransaction>,
    pub col: E::C,
    pub look_ahead: Vec<LookaheadX<E>>,
    pub condition: Condition,
}

#[async_trait]
impl<E> Loader<String> for LoaderX<E>
where
    E: EntityX,
{
    type Value = E::G;
    type Error = GrandLineErr;

    async fn load(&self, keys: &[String]) -> Res<HashMap<String, E::G>> {
        let tx = self.tx.as_ref();
        let r = E::find()
            .filter(self.col.is_in(keys))
            .filter(self.condition.clone())
            .gql_select_with_look_ahead(&self.look_ahead, self.col)?
            .all(tx)
            .await?;
        let mut map = HashMap::<String, E::G>::new();
        for g in r {
            let c = g.get_string(self.col).ok_or_else(|| MyErr::LoaderKeyNone {
                col: self.col.to_string_with_model_name(),
            })?;
            map.insert(c, g);
        }
        Ok(map)
    }
}
