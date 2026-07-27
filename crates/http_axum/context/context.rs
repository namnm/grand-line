use crate::prelude::*;
use axum::http::HeaderMap;

/// Extracts request headers from an axum HeaderMap into the context's header format.
pub trait HttpAxumContext<'a>
where
    Self: CoreContext<'a>,
{
    /// Group axum headers by name, preserving multiple values per name.
    fn axum_headers(h: &HeaderMap) -> Option<HashMap<String, Vec<String>>> {
        let mut m = HashMap::<String, Vec<String>>::new();
        for (k, v) in h {
            let k = k.as_str().to_owned();
            let v = v.to_str().unwrap_or("").to_owned();
            m.entry(k).or_default().push(v);
        }
        Some(m)
    }

    /// Read the axum HeaderMap stored in the context's request data, if any.
    fn get_headers(&self) -> Option<HashMap<String, Vec<String>>> {
        let h = self.data_opt_impl::<HeaderMap>()?;
        Self::axum_headers(h)
    }
}

impl<'a> HttpAxumContext<'a> for Context<'a> {
}
