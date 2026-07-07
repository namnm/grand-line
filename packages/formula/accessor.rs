use _core::prelude::*;
use rhai::{Dynamic, Engine, EvalAltResult, ImmutableString, Map as RhaiMap};
use tokio::runtime::Handle;

/// Sync-call track: DB accessor called from inside Rhai via block_on.
///
/// Implement this to handle db_find_one / db_find_many calls inside formula
/// scripts. Use register_db_fns to wire it into a per-eval engine.
///
/// Safe to call Handle::block_on from here regardless of the caller's tokio
/// runtime flavor: eval_formula runs the whole Rhai evaluation, including any
/// synchronous native function call triggered by the script, inside
/// spawn_blocking, so this method always executes on a dedicated blocking
/// thread, never on one of the runtime's async worker threads. No extra
/// spawn_blocking is needed in the implementation.
pub trait FormulaDbAccessor: Send + Sync {
    /// Return the first matching record, or Dynamic::UNIT when not found.
    fn find_one_sync(&self, table: &str, filter: &RhaiMap, handle: &Handle) -> Res<Dynamic>;

    /// Return all matching records.
    fn find_many_sync(&self, table: &str, filter: &RhaiMap, handle: &Handle) -> Res<Vec<Dynamic>>;
}

/// Shareable, per-eval engine configuration closure. Stored in config structs
/// and called once per eval to register functions on a fresh Rhai engine.
pub type FormulaEngineFns = Arc<dyn Fn(&mut Engine) + Send + Sync>;

/// Build a reusable function-registration closure for db_find_one and
/// db_find_many, backed by accessor. Store the result in config and pass
/// it to eval_formula via register_fns.
pub fn register_db_fns(accessor: Arc<dyn FormulaDbAccessor>) -> Arc<dyn Fn(&mut Engine) + Send + Sync> {
    Arc::new(move |engine: &mut Engine| {
        let acc = Arc::clone(&accessor);
        engine.register_fn(
            "db_find_one",
            move |table: ImmutableString, filter: RhaiMap| -> Result<Dynamic, Box<EvalAltResult>> {
                let handle = Handle::current();
                acc.find_one_sync(&table, &filter, &handle)
                    .map_err(|e| e.to_string().into())
            },
        );
        let acc = Arc::clone(&accessor);
        engine.register_fn(
            "db_find_many",
            move |table: ImmutableString, filter: RhaiMap| -> Result<Dynamic, Box<EvalAltResult>> {
                let handle = Handle::current();
                let v = acc
                    .find_many_sync(&table, &filter, &handle)
                    .map_err(|e| Box::<EvalAltResult>::from(e.to_string()))?;
                Ok(Dynamic::from(v))
            },
        );
    })
}
