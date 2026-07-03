use super::prelude::*;

#[derive(Clone)]
pub struct Filter {
    pub include_deleted: bool, // used if the client didn't pass includeDeleted
    pub condition: Condition,  // will be AND-ed with the client filter
}

impl Filter {
    pub fn add<C>(mut self, c: C) -> Self
    where
        C: IntoCondition,
    {
        self.condition = self.condition.add(c.into_condition());
        self
    }

    pub fn add_option<C>(self, c: Option<C>) -> Self
    where
        C: IntoCondition,
    {
        if let Some(c) = c {
            self.add(c)
        } else {
            self
        }
    }

    pub fn include_deleted<F>(&self, include_deleted: Option<bool>, filter: Option<&F>) -> bool
    where
        F: FilterX,
    {
        self.include_deleted || include_deleted.unwrap_or_default() || filter.is_some_and(|f| f.has_deleted_at())
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            include_deleted: false,
            condition: Condition::all(),
        }
    }
}

impl<F> From<F> for Filter
where
    F: FilterX,
{
    fn from(v: F) -> Self {
        Self {
            include_deleted: v.has_deleted_at(),
            condition: v.into_condition(),
        }
    }
}

impl<F> From<Option<F>> for Filter
where
    F: FilterX,
{
    fn from(v: Option<F>) -> Self {
        if let Some(v) = v {
            v.into()
        } else {
            Default::default()
        }
    }
}

#[derive(Clone)]
pub struct Search<O>
where
    O: OrderBy,
{
    pub filter: Filter,
    pub default_order_by: Vec<O>, // used if the client didn't request an order by
}

impl<O> Search<O>
where
    O: OrderBy,
{
    pub fn add<C>(mut self, c: C) -> Self
    where
        C: IntoCondition,
    {
        self.filter = self.filter.add(c);
        self
    }

    pub fn add_option<C>(mut self, c: Option<C>) -> Self
    where
        C: IntoCondition,
    {
        self.filter = self.filter.add_option(c);
        self
    }

    pub fn include_deleted<F>(&self, include_deleted: Option<bool>, filter: Option<&F>) -> bool
    where
        F: FilterX,
    {
        self.filter.include_deleted(include_deleted, filter)
    }
}

impl<O> Default for Search<O>
where
    O: OrderBy,
{
    fn default() -> Self {
        Self {
            filter: Default::default(),
            default_order_by: Vec::new(),
        }
    }
}

impl<O, F> From<F> for Search<O>
where
    O: OrderBy,
    F: FilterX,
{
    fn from(v: F) -> Self {
        Self {
            filter: v.into(),
            ..Default::default()
        }
    }
}

impl<O, F> From<Option<F>> for Search<O>
where
    O: OrderBy,
    F: FilterX,
{
    fn from(v: Option<F>) -> Self {
        Self {
            filter: v.into(),
            ..Default::default()
        }
    }
}

impl<O> From<Vec<O>> for Search<O>
where
    O: OrderBy,
{
    fn from(v: Vec<O>) -> Self {
        Self {
            default_order_by: v,
            ..Default::default()
        }
    }
}

impl<O, F> From<(F, Vec<O>)> for Search<O>
where
    O: OrderBy,
    F: FilterX,
{
    fn from(v: (F, Vec<O>)) -> Self {
        Self {
            default_order_by: v.1,
            ..v.0.into()
        }
    }
}

impl<O, F> From<(Option<F>, Vec<O>)> for Search<O>
where
    O: OrderBy,
    F: FilterX,
{
    fn from(v: (Option<F>, Vec<O>)) -> Self {
        Self {
            default_order_by: v.1,
            ..v.0.into()
        }
    }
}

pub type Count = Filter;
pub type Detail = Filter;
