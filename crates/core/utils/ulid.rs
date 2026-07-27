use ulid::Ulid;

/// Generates a new ULID and returns it as a lowercase string.
pub fn ulid() -> String {
    Ulid::generate().to_string().to_lowercase()
}
