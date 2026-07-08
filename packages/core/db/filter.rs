use super::prelude::*;

/// Helper trait to combine filter and `filter_extra`.
pub trait Filter<E>
where
    E: EntityX,
    Self: IntoCondition + ChainSelect<E> + Default + Serialize + Send + Sync,
{
    /// Combine filter and `filter_extra` to use in abstract methods.
    /// Should be generated in the model macro.
    fn combine_and(a: Self, b: Self) -> Self;
    /// Check if there is `deleted_at` in this filter, without the combination of nested and/or/not.
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

    /// Check if there is `deleted_at` in this filter, with the combination of nested and/or/not.
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

/// Automatically implement `FilterImpl` for Option<Filter>.
pub trait FilterImpl<E>
where
    E: EntityX,
{
    /// Helper to combine filter and `filter_extra`.
    fn combine(self, filter_extra: Self) -> Self;
    /// Check if there is `deleted_at` in this filter.
    fn has_deleted_at(&self) -> bool;
}

/// Automatically implement `FilterImpl` for Option<Filter>.
impl<E, F> FilterImpl<E> for Option<F>
where
    E: EntityX,
    F: Filter<E>,
{
    fn combine(self, filter_extra: Self) -> Self {
        match (self, filter_extra) {
            (Some(a), Some(b)) => Some(F::combine_and(a, b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }
    fn has_deleted_at(&self) -> bool {
        self.as_ref().is_some_and(Filter::has_deleted_at)
    }
}
