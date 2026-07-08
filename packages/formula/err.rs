use _core::prelude::*;

#[grand_line_err]
pub enum FormulaErr {
    #[error("formula compile error: {0}")]
    Compile(String),
    #[error("undefined variable `{0}` in formula")]
    UnknownVar(String),
    #[error("formula eval error: {0}")]
    Eval(String),
    #[error("formula result error: {0}")]
    Serialize(String),
    #[error("cyclic dependency in formula graph: {0}")]
    CyclicDep(String),
    #[error("unknown dependency `{0}` referenced in formula graph")]
    UnknownDep(String),
}
