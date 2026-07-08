use super::prelude::*;

/// Pagination `async_graphql` input struct to use in search query.
#[gql_input]
pub struct Pagination {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

/// Pagination after unwrap option into inner.
pub struct PaginationInner {
    pub offset: u64,
    pub limit: u64,
}

/// Helper trait to get offset and limit from pagination, with default and max limit.
pub trait ToPaginationInner {
    /// Helper to get offset and limit from pagination, with default and max limit.
    fn inner(self, c: &CoreConfig) -> PaginationInner;
}

/// Automatically implement `ToPaginationInner` for Pagination.
impl ToPaginationInner for Pagination {
    fn inner(self, c: &CoreConfig) -> PaginationInner {
        PaginationInner {
            offset: self.offset.unwrap_or_default(),
            limit: self
                .limit
                .map(|l| {
                    if l > c.limit_max {
                        c.limit_max
                    } else {
                        l
                    }
                })
                .unwrap_or(c.limit_default),
        }
    }
}

/// Automatically implement `ToPaginationInner` for Option<Pagination>.
impl ToPaginationInner for Option<Pagination> {
    fn inner(self, c: &CoreConfig) -> PaginationInner {
        match self {
            Some(p) => p.inner(c),
            None => PaginationInner {
                offset: 0,
                limit: c.limit_default,
            },
        }
    }
}
