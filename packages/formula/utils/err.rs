use _core::prelude::*;

#[grand_line_err]
pub enum MyErr {
    // ========================================================================
    // client errors
    //

    // ========================================================================
    // server errors
    //
    #[error("formula compile error: {inner}")]
    Compile {
        inner: String,
    },
    #[error("undefined variable {name} in formula")]
    UnknownVar {
        name: String,
    },
    #[error("formula eval error: {inner}")]
    Eval {
        inner: String,
    },
    #[error("formula result error: {inner}")]
    Serialize {
        inner: String,
    },
    #[error("cyclic dependency in formula graph, involved nodes: {nodes}")]
    CyclicDep {
        nodes: String,
    },
    #[error("node {node} in formula graph depends on unknown node {dep}")]
    UnknownDep {
        node: String,
        dep: String,
    },
}
