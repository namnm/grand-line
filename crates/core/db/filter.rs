use super::prelude::*;

/// The serde field names a filter accepts. Generated filters deserialize
/// leniently (`#[serde(default)]`, unknown keys silently dropped) because
/// client input is coerced by graphql first. A filter arriving from the
/// authorization boundary (see authz_row) is validated against this list
/// instead, so a typo in policy data cannot silently produce an empty filter,
/// i.e. an empty WHERE, i.e. every row. The model macro generates the impl.
pub trait FilterKeys {
    fn known_keys() -> &'static [&'static str];
}

// ---------------------------------------------------------------------------
// FilterX, condition plus include_deleted helpers
// ---------------------------------------------------------------------------

/// Helper trait to combine filter and filter_extra.
pub trait FilterX
where
    Self: IntoCondition + ChainSelect<Self::E> + Default + Serialize + Send + Sync,
{
    type E: EntityX;
    /// Combine filter and filter_extra to use in abstract methods.
    /// Should be generated in the model macro.
    fn combine_and(a: Self, b: Self) -> Self;
    /// Check if there is deleted_at in this filter, without the combination of nested and/or/not.
    /// Should be generated in the model macro.
    fn has_deleted_at_without_nested(&self) -> bool;
    /// Get and to use in abstract methods.
    /// Should be generated in the model macro.
    fn get_and(&self) -> Option<Vec<Self>>;
    /// Get or to use in abstract methods.
    /// Should be generated in the model macro.
    fn get_or(&self) -> Option<Vec<Self>>;
    /// Get not to use in abstract methods.
    /// Should be generated in the model macro.
    fn get_not(&self) -> Option<Self>;
    // Turn this filter into UpdateMany.
    fn into_update(self) -> UpdateMany<Self::E> {
        Self::E::update_many().filter(self.into_condition())
    }
    // Turn this filter into DeleteMany.
    fn into_delete(self) -> DeleteMany<Self::E> {
        Self::E::delete_many().filter(self.into_condition())
    }
}

// ---------------------------------------------------------------------------
// FilterXImpl, deleted_at check with nested and/or/not combined in
// ---------------------------------------------------------------------------

/// Extension trait exposing has_deleted_at with nested and/or/not combined in,
/// implemented below for any FilterX and for Option<FilterX>.
pub trait FilterXImpl {
    /// Check if there is deleted_at in this filter.
    fn has_deleted_at(&self) -> bool;
}

/// Automatically implement FilterXImpl for any type implementing FilterX.
impl<F> FilterXImpl for F
where
    Self: FilterX,
{
    /// Check if there is deleted_at in this filter, with the combination of nested and/or/not.
    fn has_deleted_at(&self) -> bool {
        if self.has_deleted_at_without_nested() {
            return true;
        }
        if let Some(and) = self.get_and()
            && and.iter().any(Self::has_deleted_at)
        {
            return true;
        }
        if let Some(or) = self.get_or()
            && or.iter().any(Self::has_deleted_at)
        {
            return true;
        }
        if let Some(not) = self.get_not()
            && not.has_deleted_at()
        {
            return true;
        }
        false
    }
}

/// Automatically implement FilterXImpl for Option<FilterX>.
impl<F> FilterXImpl for Option<F>
where
    F: FilterX,
{
    fn has_deleted_at(&self) -> bool {
        self.as_ref().is_some_and(|v| v.has_deleted_at())
    }
}
