use super::prelude::*;

/// Abstract extra `QueryFilter` methods implementation.
pub trait QueryFilterX<E>
where
    E: EntityX,
    Self: QueryFilter,
{
    /// Filter with condition id eq.
    fn filter_by_id(self, id: &str) -> Self {
        self.filter(E::cond_id(id))
    }
    /// Filter exclude deleted if there is `deleted_at` column.
    fn exclude_deleted(self) -> Self {
        match E::cond_exclude_deleted() {
            Some(c) => self.filter(c),
            None => self,
        }
    }
}

/// Automatically implement for Select<E>.
impl<E> QueryFilterX<E> for Select<E> where E: EntityX {}
/// Automatically implement for `DeleteMany`<E>.
impl<E> QueryFilterX<E> for DeleteMany<E> where E: EntityX {}
/// Automatically implement for `UpdateMany`<E>.
impl<E> QueryFilterX<E> for UpdateMany<E> where E: EntityX {}
