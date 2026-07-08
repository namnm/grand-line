use heck::{ToLowerCamelCase as _, ToPascalCase as _, ToSnakeCase as _};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{Attribute, Item, ItemFn, Meta, Token, parse_file, punctuated::Punctuated};

// ============================================================================
// Public API

/// Scan `src/` of the current crate and generate `$OUT_DIR/grand_line_schema.rs`
/// containing `pub struct Query(...)` and `pub struct Mutation(...)`.
///
/// Call from `build.rs`:
/// ```rust
/// fn main() {
///     grand_line_build::generate_schema();
/// }
/// ```
///
/// Then in your crate:
/// ```rust
/// include!(concat!(env!("OUT_DIR"), "/grand_line_schema.rs"));
/// ```
pub fn generate_schema() {
    SchemaBuilder::new().scan("src").generate();
}

/// Builder for more control: multiple source dirs and extra merged types.
///
/// ```rust
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
}

impl SchemaBuilder {
    pub const fn new() -> Self {
        Self {
            dirs: vec![],
            extra_query: vec![],
            extra_mutation: vec![],
        }
    }

    /// Add a source directory to scan (relative to `CARGO_MANIFEST_DIR`).
    pub fn scan(mut self, dir: &str) -> Self {
        self.dirs.push(dir.to_owned());
        self
    }

    /// Prepend an extra type to `Query` (e.g. `"AuthMergedQuery"`).
    pub fn extra_query(mut self, ty: &str) -> Self {
        self.extra_query.push(ty.to_owned());
        self
    }

    /// Prepend an extra type to `Mutation` (e.g. `"AuthMergedMutation<User>"`).
    pub fn extra_mutation(mut self, ty: &str) -> Self {
        self.extra_mutation.push(ty.to_owned());
        self
    }

    /// Scan all configured dirs, compute resolver struct names, and write
    /// `$OUT_DIR/grand_line_schema.rs`.
    pub fn generate(self) {
        let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("CARGO_MANIFEST_DIR not set: {e}");
                eprintln!("cargo:error=grand_line_build: {msg}");
                return;
            }
        };
        let out_dir = match env::var("OUT_DIR") {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("OUT_DIR not set: {e}");
                eprintln!("cargo:error=grand_line_build: {msg}");
                return;
            }
        };

        let mut query_types = self.extra_query;
        let mut mutation_types = self.extra_mutation;

        for rel_dir in &self.dirs {
            let abs_dir = Path::new(&manifest_dir).join(rel_dir);
            scan_dir(&abs_dir, &mut query_types, &mut mutation_types);
            println!("cargo:rerun-if-changed={}", abs_dir.display());
        }

        let query_types = dedup_warn(query_types, "query");
        let mutation_types = dedup_warn(mutation_types, "mutation");

        let code = generate(&query_types, &mutation_types);
        let out_path = PathBuf::from(&out_dir).join("grand_line_schema.rs");
        if let Err(e) = fs::write(&out_path, code) {
            let msg = format!("failed to write grand_line_schema.rs: {e}");
            eprintln!("cargo:error=grand_line_build: {msg}");
        }
    }
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// File scanning - uses syn for accurate AST parsing

fn scan_dir(dir: &Path, query_types: &mut Vec<String>, mutation_types: &mut Vec<String>) {
    if !dir.exists() {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, query_types, mutation_types);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let content = fs::read_to_string(&path).unwrap_or_default();
            scan_file(&content, query_types, mutation_types);
        }
    }
}

fn scan_file(content: &str, query_types: &mut Vec<String>, mutation_types: &mut Vec<String>) {
    let Ok(file) = parse_file(content) else {
        return;
    };
    scan_items(&file.items, query_types, mutation_types);
}

fn scan_items(items: &[Item], query_types: &mut Vec<String>, mutation_types: &mut Vec<String>) {
    for item in items {
        match item {
            Item::Fn(fn_item) => scan_fn(fn_item, query_types, mutation_types),
            // Recurse into inline mod blocks.
            Item::Mod(m) => {
                if let Some((_, items)) = &m.content {
                    scan_items(items, query_types, mutation_types);
                }
            }
            _ => {}
        }
    }
}

fn scan_fn(fn_item: &ItemFn, query_types: &mut Vec<String>, mutation_types: &mut Vec<String>) {
    let fn_name = fn_item.sig.ident.to_string();
    let resolver_attrs: Vec<(String, &'static str, String)> =
        fn_item.attrs.iter().filter_map(detect_resolver_attr).collect();

    if resolver_attrs.len() > 1 {
        let msg =
            format!("`{fn_name}` has multiple resolver attributes; only one resolver attribute per function is valid");
        println!("cargo:warning=grand_line_build: {msg}");
    }

    if let Some((crud, operation, model)) = resolver_attrs.into_iter().next() {
        if !crud.is_empty() && model.is_empty() {
            let msg = format!(
                "#[{crud}] on `{fn_name}` is missing a model argument (expected #[{crud}(Model, ...)]); skipped",
            );
            println!("cargo:warning=grand_line_build: {msg}");
            return;
        }
        let struct_name = resolver_struct_name(&fn_name, &crud, &model, operation);
        if operation == "query" {
            query_types.push(struct_name);
        } else {
            mutation_types.push(struct_name);
        }
    }
}

// ============================================================================
// Attribute detection

const CRUD_MACROS: &[(&str, &str, &str)] = &[
    ("search", "search", "query"),
    ("count", "count", "query"),
    ("detail", "detail", "query"),
    ("create", "create", "mutation"),
    ("update", "update", "mutation"),
    ("delete", "delete", "mutation"),
];

const MANUAL_MACROS: &[(&str, &str)] = &[("query", "query"), ("mutation", "mutation")];

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
// Handles `#[search(Todo)]`, `#[update(Todo, resolver_inputs)]`, etc.
fn first_arg_ident(attr: &Attribute) -> Option<String> {
    let args = attr
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .ok()?;

    match args.into_iter().next()? {
        Meta::Path(p) => p.get_ident().map(|v| v.to_string()),
        Meta::List(l) => l.path.get_ident().map(|v| v.to_string()),
        Meta::NameValue(nv) => nv.path.get_ident().map(|v| v.to_string()),
    }
}

// ============================================================================
// Name computation - mirrors resolver_ty_item.rs::init exactly

fn resolver_struct_name(fn_name: &str, crud: &str, model: &str, operation: &str) -> String {
    let gql_name = if fn_name == "resolver" && !crud.is_empty() {
        format!("{model}_{crud}").to_lower_camel_case()
    } else {
        fn_name.to_lower_camel_case()
    };
    let name = gql_name.to_snake_case();
    format!("{name}_{operation}").to_pascal_case()
}

// ============================================================================
// Deduplication

fn dedup_warn(types: Vec<String>, kind: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    types
        .into_iter()
        .filter(|t| {
            if seen.insert(t.clone()) {
                true
            } else {
                let msg = format!("duplicate {kind} type `{t}`; keeping one");
                println!("cargo:warning=grand_line_build: {msg}");
                false
            }
        })
        .collect()
}

// ============================================================================
// Code generation

fn generate(query_types: &[String], mutation_types: &[String]) -> String {
    let mut out: Vec<String> = vec![];
    if !query_types.is_empty() {
        gen_merged_object(&mut out, "Query", query_types);
    }
    if !mutation_types.is_empty() {
        gen_merged_object(&mut out, "Mutation", mutation_types);
    }
    out.join("\n")
}

fn gen_merged_object(out: &mut Vec<String>, name: &str, types: &[String]) {
    out.push("#[derive(Default, MergedObject)]".to_owned());
    let op = format!("pub struct {}({});", name, types.join(","));
    out.push(op);
}
