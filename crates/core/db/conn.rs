use super::prelude::*;

/// Message for a statement issued after the request transaction was already
/// committed or rolled back, e.g. from a dataloader batch left running by a
/// cancelled resolver.
const TX_GONE: &str = "the request transaction is already closed";

/// The connection a resolver reads and writes through, either the request
/// transaction or a pooled connection.
///
/// A borrow, never an owning handle: committing consumes the transaction, so
/// anything holding one past the end of the request would block the commit. Tying
/// this to the lifetime of GrandLineData makes that unrepresentable rather than a
/// rule to remember. The transaction is locked per statement, which is the same
/// serialization sea_orm already does inside DatabaseTransaction, and a pooled
/// read locks nothing at all.
pub struct ConnX<'a> {
    db: &'a DatabaseConnection,
    tx: Option<&'a Mutex<Option<DatabaseTransaction>>>,
}

impl<'a> ConnX<'a> {
    /// A pooled connection, for a read that needs no transaction.
    pub const fn db(db: &'a DatabaseConnection) -> Self {
        Self {
            db,
            tx: None,
        }
    }

    /// The request transaction, with db kept alongside only to answer for the
    /// database backend, which is a property of the pool and never of one
    /// transaction, and which sea_orm asks for from a sync method.
    pub const fn tx(db: &'a DatabaseConnection, tx: &'a Mutex<Option<DatabaseTransaction>>) -> Self {
        Self {
            db,
            tx: Some(tx),
        }
    }

    /// Whether this is the request transaction rather than a pooled connection.
    pub const fn is_tx(&self) -> bool {
        self.tx.is_some()
    }
}

/// Runs the statement against whichever connection this is, holding the
/// transaction lock only for as long as the statement itself.
macro_rules! run {
    ($self:expr, $c:ident, $body:expr) => {
        match $self.tx {
            Some(m) => {
                let guard = m.lock().await;
                let $c = guard
                    .as_ref()
                    .ok_or_else(|| DbErr::Conn(RuntimeErr::Internal(TX_GONE.to_owned())))?;
                let r = $body;
                // Explicit, so the next statement on this transaction is not waiting
                // on a guard the compiler would otherwise keep to the end of the arm.
                drop(guard);
                r
            }
            None => {
                let $c = $self.db;
                $body
            }
        }
    };
}

#[async_trait]
impl ConnectionTrait for ConnX<'_> {
    fn get_database_backend(&self) -> DbBackend {
        self.db.get_database_backend()
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        run!(self, c, c.execute_raw(stmt).await)
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        run!(self, c, c.execute_unprepared(sql).await)
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        run!(self, c, c.query_one_raw(stmt).await)
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        run!(self, c, c.query_all_raw(stmt).await)
    }
}
