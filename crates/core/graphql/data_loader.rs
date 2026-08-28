use super::prelude::*;
use dataloader::Loader;

/// Batch loader for entity E, keyed by the string value of col.
///
/// Holds a Weak of the request data rather than a connection: async_graphql runs a
/// batch in a task spawned off the resolver awaiting it, and that task keeps the
/// loader alive for a moment after it has already handed the rows back. Anything
/// stronger here would outlive the request and block its commit.
pub struct LoaderX<E>
where
    E: EntityX,
{
    pub gl: Weak<GrandLineData>,
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
        // Upgrading only for the duration of the batch, a batch left running by a
        // cancelled resolver finds the request already gone and errors here rather
        // than reaching for a connection nothing owns any more.
        let gl = self.gl.upgrade().ok_or(MyErr::LoaderTxGone)?;
        let db = &gl.db().await?;
        let r = E::find()
            .filter(self.col.is_in(keys))
            .filter(self.condition.clone())
            .gql_select_with_look_ahead(&self.look_ahead, self.col)?
            .all(db)
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
