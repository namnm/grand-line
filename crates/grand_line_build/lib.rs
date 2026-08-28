use heck::{ToLowerCamelCase as _, ToPascalCase as _, ToSnakeCase as _};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use syn::{Attribute, Item, ItemFn, Meta, parse_file, punctuated::Punctuated, token::Comma};

// ----------------------------------------------------------------------------
// Failure reporting
// ----------------------------------------------------------------------------

/// Report a fatal build script error and stop the build.
/// cargo::error= with two colons (rust 1.84) is a real directive read from stdout,
/// the cargo:error= on stderr this used to print was read by nothing, so every
/// failure here let the build pass with an empty or stale schema.
#[allow(
    clippy::exit,
    reason = "a build script cannot return an error, exiting is how it fails the build"
)]
fn fail(msg: &str) -> ! {
    println!("cargo::error=grand_line_build: {msg}");
    process::exit(1);
}

// ----------------------------------------------------------------------------
// Public API
// ----------------------------------------------------------------------------

/// Scan src/ of the current crate and generate $OUT_DIR/grand_line_schema.rs
/// containing pub struct Query(...) and pub struct Mutation(...).
///
/// Call from build.rs, no_run because it only works with cargo's build script
/// env vars set, and now fails the build rather than silently doing nothing:
/// ```no_run
/// fn main() {
///     grand_line_build::generate_schema();
/// }
/// ```
///
/// Then in your crate:
/// ```ignore
/// include!(concat!(env!("OUT_DIR"), "/grand_line_schema.rs"));
/// ```
pub fn generate_schema() {
    SchemaBuilder::new().scan("src").generate();
}

/// Builder for more control: multiple source dirs and extra merged types.
///
/// ```no_run
/// grand_line_build::SchemaBuilder::new()
///     .scan("src")
///     .scan("../other_crate/src")
///     .extra_query("AuthMergedQuery")
///     .extra_mutation("AuthMergedMutation<User>")
///     .generate();
/// ```
pub struct SchemaBuilder {
    dirs: Vec<String>,
    extra_query: Vec<String>,
    extra_mutation: Vec<String>,
    extra_subscription: Vec<String>,
}

impl SchemaBuilder {
    /// Creates an empty builder with no scan dirs or extra query/mutation types.
    pub const fn new() -> Self {
        Self {
            dirs: vec![],
            extra_query: vec![],
            extra_mutation: vec![],
            extra_subscription: vec![],
        }
    }

    /// Add a source directory to scan (relative to CARGO_MANIFEST_DIR).
    pub fn scan(mut self, dir: &str) -> Self {
        self.dirs.push(dir.to_owned());
        self
    }

    /// Prepend an extra type to Query (e.g. "AuthMergedQuery").
    pub fn extra_query(mut self, ty: &str) -> Self {
        self.extra_query.push(ty.to_owned());
        self
    }

    /// Prepend an extra type to Mutation (e.g. "AuthMergedMutation<User>").
    pub fn extra_mutation(mut self, ty: &str) -> Self {
        self.extra_mutation.push(ty.to_owned());
        self
    }

    /// Prepend an extra type to Subscription (e.g. "FileMergedSubscription").
    pub fn extra_subscription(mut self, ty: &str) -> Self {
        self.extra_subscription.push(ty.to_owned());
        self
    }

    /// Scan all configured dirs, compute resolver struct names, and write
    /// $OUT_DIR/grand_line_schema.rs.
    pub fn generate(self) {
        let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("CARGO_MANIFEST_DIR not set: {e}");
                fail(&msg);
            }
        };
        let out_dir = match env::var("OUT_DIR") {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("OUT_DIR not set: {e}");
                fail(&msg);
            }
        };

        let dirs = match resolve_dirs(Path::new(&manifest_dir), &self.dirs) {
            Ok(v) => v,
            Err(msg) => fail(&msg),
        };

        let mut roots = Roots {
            query: self.extra_query,
            mutation: self.extra_mutation,
            subscription: self.extra_subscription,
        };

        for abs_dir in &dirs {
            scan_dir(abs_dir, &mut roots);
            let abs_dir = abs_dir.display();
            println!("cargo:rerun-if-changed={abs_dir}");
        }

        roots.query = dedup_warn(roots.query, "query");
        roots.mutation = dedup_warn(roots.mutation, "mutation");
        roots.subscription = dedup_warn(roots.subscription, "subscription");

        let code = generate(&roots);
        let out_path = PathBuf::from(&out_dir).join("grand_line_schema.rs");
        if let Err(e) = fs::write(&out_path, code) {
            let msg = format!("failed to write grand_line_schema.rs: {e}");
            fail(&msg);
        }
    }
}

/// Resolve every configured scan dir against manifest_dir, erroring on the first
/// one that is not a directory. A stale path used to scan nothing and keep the
/// build green, which is how a refactor left several dead scan entries behind.
pub fn resolve_dirs(manifest_dir: &Path, dirs: &[String]) -> Result<Vec<PathBuf>, String> {
    dirs.iter()
        .map(|rel| {
            let abs = manifest_dir.join(rel);
            if abs.is_dir() {
                return Ok(abs);
            }
            let display = abs.display();
            let msg = format!("scan dir not found: {rel} (resolved to {display})");
            Err(msg)
        })
        .collect()
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------------------
// File scanning - uses syn for accurate AST parsing
// ----------------------------------------------------------------------------

fn scan_dir(dir: &Path, out: &mut Roots) {
    if !dir.exists() {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let content = fs::read_to_string(&path).unwrap_or_default();
            scan_file(&content, out);
        }
    }
}

fn scan_file(content: &str, out: &mut Roots) {
    let Ok(file) = parse_file(content) else {
        return;
    };
    scan_items(&file.items, out);
}

fn scan_items(items: &[Item], out: &mut Roots) {
    for item in items {
        match item {
            Item::Fn(ifn) => scan_fn(ifn, out),
            // Recurse into inline mod blocks.
            Item::Mod(m) => {
                if let Some((_, items)) = &m.content {
                    scan_items(items, out);
                }
            }
            _ => {}
        }
    }
}

fn scan_fn(ifn: &ItemFn, out: &mut Roots) {
    let f = ifn.sig.ident.to_string();
    let resolver_attrs = ifn
        .attrs
        .iter()
        .filter_map(detect_resolver_attr)
        .collect::<Vec<(String, &'static str, String)>>();

    if resolver_attrs.len() > 1 {
        let msg = format!("{f} has multiple resolver attributes; only one resolver attribute per function is valid");
        println!("cargo:warning=grand_line_build: {msg}");
    }

    if let Some((crud, operation, model)) = resolver_attrs.into_iter().next() {
        if !crud.is_empty() && model.is_empty() {
            let msg = format!("#[{crud}] on {f} is missing a model argument (expected #[{crud}(Model, ...)]); skipped");
            println!("cargo:warning=grand_line_build: {msg}");
            return;
        }
        let struk = resolver_struct_name(&f, &crud, &model, operation);
        match operation {
            "query" => out.query.push(struk),
            "subscription" => out.subscription.push(struk),
            _ => out.mutation.push(struk),
        }
    }
}

// ----------------------------------------------------------------------------
// Attribute detection
// ----------------------------------------------------------------------------

const CRUD_MACROS: &[(&str, &str, &str)] = &[
    ("search", "search", "query"),
    ("count", "count", "query"),
    ("detail", "detail", "query"),
    ("create", "create", "mutation"),
    ("update", "update", "mutation"),
    ("delete", "delete", "mutation"),
    ("subscribe", "changed", "subscription"),
];

const MANUAL_MACROS: &[(&str, &str)] = &[("query", "query"), ("mutation", "mutation")];

/// Root resolver struct names collected from one scan, one bucket per operation.
#[derive(Default)]
struct Roots {
    query: Vec<String>,
    mutation: Vec<String>,
    subscription: Vec<String>,
}

fn detect_resolver_attr(attr: &Attribute) -> Option<(String, &'static str, String)> {
    let macro_name = attr.path().get_ident()?.to_string();

    for (attr_name, crud, operation) in CRUD_MACROS {
        if macro_name == *attr_name {
            let model = first_arg_ident(attr).unwrap_or_default();
            return Some((crud.to_string(), operation, model));
        }
    }

    for (attr_name, operation) in MANUAL_MACROS {
        if macro_name == *attr_name {
            return Some((String::new(), operation, String::new()));
        }
    }

    None
}

// Parse the first argument of an attribute as an identifier.
// Handles #[search(Todo)], #[update(Todo, resolver_inputs)], etc.
fn first_arg_ident(attr: &Attribute) -> Option<String> {
    let args = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated).ok()?;

    match args.into_iter().next()? {
        Meta::Path(p) => p.get_ident().map(|v| v.to_string()),
        Meta::List(l) => l.path.get_ident().map(|v| v.to_string()),
        Meta::NameValue(nv) => nv.path.get_ident().map(|v| v.to_string()),
    }
}

// ----------------------------------------------------------------------------
// Name computation - mirrors resolver_ty_item.rs::init exactly
// ----------------------------------------------------------------------------

fn resolver_struct_name(f: &str, crud: &str, model: &str, operation: &str) -> String {
    let gql_name = if f == "resolver" && !crud.is_empty() {
        format!("{model}_{crud}").to_lower_camel_case()
    } else {
        f.to_lower_camel_case()
    };
    let name = gql_name.to_snake_case();
    format!("{name}_{operation}").to_pascal_case()
}

// ----------------------------------------------------------------------------
// Deduplication
// ----------------------------------------------------------------------------

fn dedup_warn(types: Vec<String>, kind: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    types
        .into_iter()
        .filter(|t| {
            if seen.insert(t.clone()) {
                true
            } else {
                let msg = format!("duplicate {kind} type {t}, keeping one");
                println!("cargo:warning=grand_line_build: {msg}");
                false
            }
        })
        .collect()
}

// ----------------------------------------------------------------------------
// Code generation
// ----------------------------------------------------------------------------

fn generate(roots: &Roots) -> String {
    let mut out: Vec<String> = vec![];
    if !roots.query.is_empty() {
        gen_merged_object(&mut out, "Query", &roots.query, "MergedObject");
    }
    if !roots.mutation.is_empty() {
        gen_merged_object(&mut out, "Mutation", &roots.mutation, "MergedObject");
    }
    if !roots.subscription.is_empty() {
        gen_merged_object(&mut out, "Subscription", &roots.subscription, "MergedSubscription");
    }
    out.join("\n")
}

fn gen_merged_object(out: &mut Vec<String>, name: &str, types: &[String], derive: &str) {
    let types = types.join(",");
    out.push(format!("#[derive(Default, {derive})]"));
    out.push(format!("pub struct {name}({types});"));
}
