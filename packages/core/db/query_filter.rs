use super::prelude::*;

/// Abstract extra QueryFilter methods implementation.
pub trait QueryFilterX
where
    Self: QueryFilter,
{
    type E: EntityX;

    fn filter_option<C>(self, c: Option<C>) -> Self
    where
        C: IntoCondition,
    {
        if let Some(c) = c {
            self.filter(c)
        } else {
            self
        }
    }

    /// Filter with condition id eq.
    fn filter_by_id(self, id: &str) -> Self {
        self.filter(Self::E::cond_id(id))
    }

    /// Filter exclude deleted if there is deleted_at column.
    /// To easier to chain with user input include_deleted.
    fn include_deleted(self, include_deleted: bool) -> Self {
        if include_deleted {
            self
        } else {
            match Self::E::cond_exclude_deleted() {
                Some(c) => self.filter(c),
                None => self,
            }
        }
    }
}

/// Automatically implement for Select<E>.
impl<E> QueryFilterX for Select<E>
where
    E: EntityX,
{
    type E = E;
}
/// Automatically implement for DeleteMany<E>.
impl<E> QueryFilterX for DeleteMany<E>
where
    E: EntityX,
{
    type E = E;
}
/// Automatically implement for UpdateMany<E>.
impl<E> QueryFilterX for UpdateMany<E>
where
    E: EntityX,
{
    type E = E;
}
