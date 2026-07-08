use crate::err::FormulaErr;
use crate::resolver::{FormulaResolver, NowResolver};
use _core::prelude::*;
use std::collections::VecDeque;

/// A single node in a `FormulaDepGraph`.
///
/// `name` is the Rhai scope variable this node produces.
/// `deps` lists the names of other nodes whose output this resolver needs;
/// those values will be available in `ctx.resolved` when `resolver.resolve`
/// is called.
#[derive(Clone)]
pub struct FormulaDepNode {
    pub name: String,
    pub deps: Vec<String>,
    pub resolver: Arc<dyn FormulaResolver>,
}

impl FormulaDepNode {
    pub fn new(
        name: impl Into<String>,
        deps: impl IntoIterator<Item = impl Into<String>>,
        resolver: impl FormulaResolver + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            deps: deps.into_iter().map(Into::into).collect(),
            resolver: Arc::new(resolver),
        }
    }
}

/// A validated dependency graph for formula resolvers.
///
/// Built once (e.g. in `AuthzConfig`) and reused across evals.
/// Cycle detection and topological ordering happen at construction time;
/// `eval_formula` just iterates `topo_order` and calls each resolver.
#[derive(Clone)]
pub struct FormulaDepGraph {
    nodes: HashMap<String, FormulaDepNode>,
    /// Node names in topological evaluation order (deps before dependents).
    topo_order: Vec<String>,
}

impl FormulaDepGraph {
    /// Build and validate the graph. Returns `Err(FormulaErr::CyclicDep)` when
    /// a cycle is detected, or `Err(FormulaErr::UnknownDep)` when a dep name
    /// is not present among the nodes.
    pub fn new(nodes: impl IntoIterator<Item = FormulaDepNode>) -> Result<Self, FormulaErr> {
        let nodes: HashMap<String, FormulaDepNode> = nodes.into_iter().map(|n| (n.name.clone(), n)).collect();
        let topo_order = topo_sort(&nodes)?;
        Ok(Self {
            nodes,
            topo_order,
        })
    }

    /// Empty graph with no resolvers (no external variables available).
    pub fn empty() -> Self {
        Self {
            nodes: HashMap::new(),
            topo_order: vec![],
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.nodes.contains_key(name)
    }

    pub fn topo_order(&self) -> &[String] {
        &self.topo_order
    }

    pub fn get_node(&self, name: &str) -> Option<&FormulaDepNode> {
        self.nodes.get(name)
    }

    /// Graph seeded with a single `now` node (UTC milliseconds since epoch).
    pub fn with_now() -> Self {
        let node = FormulaDepNode::new("now", [] as [&str; 0], NowResolver);
        let topo_order = vec![node.name.clone()];
        let nodes = HashMap::from([(node.name.clone(), node)]);
        Self {
            nodes,
            topo_order,
        }
    }
}

impl Default for FormulaDepGraph {
    fn default() -> Self {
        Self::with_now()
    }
}

/// Kahn's algorithm: topological sort + cycle detection in O(V+E).
fn topo_sort(nodes: &HashMap<String, FormulaDepNode>) -> Result<Vec<String>, FormulaErr> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for (name, node) in nodes {
        for dep in &node.deps {
            if !nodes.contains_key(dep.as_str()) {
                return Err(FormulaErr::UnknownDep(dep.clone()));
            }
            adj.entry(dep.as_str()).or_default().push(name.as_str());
        }
    }

    // In-degree = number of deps each node has (edges pointing into it).
    let mut in_degree: HashMap<&str, usize> = nodes.iter().map(|(k, v)| (k.as_str(), v.deps.len())).collect();

    let mut queue: VecDeque<&str> = in_degree.iter().filter(|&(_, &d)| d == 0).map(|(k, _)| *k).collect();

    let mut order: Vec<String> = Vec::with_capacity(nodes.len());
    while let Some(name) = queue.pop_front() {
        order.push(name.to_owned());
        for &dep in adj.get(name).into_iter().flatten() {
            if let Some(d) = in_degree.get_mut(dep) {
                *d -= 1;
                if *d == 0 {
                    queue.push_back(dep);
                }
            }
        }
    }

    if order.len() != nodes.len() {
        let cycle: Vec<&str> = in_degree.iter().filter(|&(_, &d)| d > 0).map(|(k, _)| *k).collect();
        return Err(FormulaErr::CyclicDep(cycle.join(", ")));
    }
    Ok(order)
}
