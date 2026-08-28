use grand_line_build::resolve_dirs;
use pretty_assertions::assert_eq as pretty_eq;
use std::path::Path;

// ---------------------------------------------------------------------------
// Scan dirs are validated, a stale one used to scan nothing and stay green
// ---------------------------------------------------------------------------

#[test]
fn existing_dir_resolves_against_the_manifest_dir() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dirs = vec!["tests".to_owned()];

    pretty_eq!(
        resolve_dirs(manifest, &dirs),
        Ok(vec![manifest.join("tests")]),
        "an existing scan dir should resolve to the manifest dir joined with it",
    );
}

#[test]
fn missing_dir_is_an_error_naming_the_path() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dirs = vec!["../commerce/observers".to_owned()];

    let e = resolve_dirs(manifest, &dirs).err().unwrap_or_default();

    pretty_eq!(
        e.contains("../commerce/observers"),
        true,
        "error should name the configured dir so a stale entry is findable",
    );
}

#[test]
fn one_missing_dir_fails_the_whole_set() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dirs = vec!["tests".to_owned(), "walternate".to_owned()];

    pretty_eq!(
        resolve_dirs(manifest, &dirs).is_err(),
        true,
        "a set holding one missing dir should not resolve",
    );
}
