use ulid::Ulid;

/// Generates a new ULID and returns it as a lowercase string.
pub fn ulid() -> String {
    Ulid::new().to_string().to_lowercase()
}
