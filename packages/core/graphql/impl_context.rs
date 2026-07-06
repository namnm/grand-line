use super::prelude::*;

/// Shortcuts so we can directly use the methods from trait definition instead of impl.
pub trait BaseImplContext<'a> {
    /// Shortcuts so we can directly use the methods from trait definition instead of impl.
    fn data_impl<D>(&self) -> GraphQLRes<&'a D>
    where
        D: Any + Send + Sync;
    /// Shortcuts so we can directly use the methods from trait definition instead of impl.
    fn data_opt_impl<D>(&self) -> Option<&'a D>
    where
        D: Any + Send + Sync;
    /// Shortcuts so we can directly use the methods from trait definition instead of impl.
    fn data_unchecked_impl<D>(&self) -> &'a D
    where
        D: Any + Send + Sync;
}

impl<'a> BaseImplContext<'a> for Context<'a> {
    fn data_impl<D>(&self) -> GraphQLRes<&'a D>
    where
        D: Any + Send + Sync,
    {
        self.data()
    }

    fn data_opt_impl<D>(&self) -> Option<&'a D>
    where
        D: Any + Send + Sync,
    {
        self.data_opt()
    }

    fn data_unchecked_impl<D>(&self) -> &'a D
    where
        D: Any + Send + Sync,
    {
        self.data_unchecked()
    }
}

impl<'a> BaseImplContext<'a> for ExtensionContext<'a> {
    fn data_impl<D>(&self) -> GraphQLRes<&'a D>
    where
        D: Any + Send + Sync,
    {
        self.data()
    }

    fn data_opt_impl<D>(&self) -> Option<&'a D>
    where
        D: Any + Send + Sync,
    {
        self.data_opt()
    }

    fn data_unchecked_impl<D>(&self) -> &'a D
    where
        D: Any + Send + Sync,
    {
        self.data_unchecked()
    }
}

/// Shortcuts so we can directly use the methods from trait definition instead of impl.
pub trait ImplContext<'a>
where
    Self: BaseImplContext<'a>,
{
    /// Shortcuts so we can directly use the methods from trait definition instead of impl.
    fn field_impl(&self) -> SelectionField<'_>;
    /// Shortcuts so we can directly use the methods from trait definition instead of impl.
    fn path_node_impl(&self) -> Option<QueryPathNode<'a>>;
    /// Shortcuts so we can directly use the methods from trait definition instead of impl.
    fn append_http_header_impl(&self, k: &'static str, v: &str) -> bool;

    fn field_path_without_number_index(&self) -> String {
        let Some(node) = self.path_node_impl() else {
            return self.field_impl().name().to_owned();
        };
        node.to_string_vec()
            .into_iter()
            .filter(|s| s.parse::<usize>().is_err())
            .collect::<Vec<_>>()
            .join(".")
    }
}

impl<'a> ImplContext<'a> for Context<'a> {
    fn field_impl(&self) -> SelectionField<'_> {
        self.field()
    }
    fn path_node_impl(&self) -> Option<QueryPathNode<'a>> {
        self.path_node
    }
    fn append_http_header_impl(&self, k: &'static str, v: &str) -> bool {
        self.append_http_header(k, v)
    }
}
