use super::prelude::*;

// ============================================================================
// Operation type markers

pub struct AmCreate;
pub struct AmUpdate;
pub struct AmSoftDelete;

// ============================================================================
// Wrapper struct
// T = operation type (AmCreate | AmUpdate | AmSoftDelete)
// E = entity (EntityX)
// A = sea-orm ActiveModel

pub struct AmWrapper<T, E, A> {
    pub(crate) am: A,
    _phantom: PhantomData<(T, E)>,
}

impl<T, E, A> AmWrapper<T, E, A> {
    pub const fn new(am: A) -> Self {
        Self {
            am,
            _phantom: PhantomData,
        }
    }
}
