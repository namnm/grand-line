use super::prelude::*;

/// Access to the database connection stored in GrandLineData.
#[async_trait]
pub trait DbContext<'a>
where
    Self: GrandLineDataContext<'a>,
{
    /// Shortcut to get db connection from grand line data.
    async fn db(&self) -> Res<&'a DatabaseConnection> {
        let db = self.grand_line()?.db.as_ref();
        Ok(db)
    }
}

#[async_trait]
impl<'a> DbContext<'a> for Context<'a> {
}
