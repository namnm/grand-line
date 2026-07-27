use super::prelude::*;

/// Helper trait to abstract extra methods into sea_orm column.
pub trait ColumnX
where
    Self: ColumnTrait,
{
    type E: EntityX;
    /// Format this column as model name and column name joined by a dot, e.g. task.title.
    fn to_string_with_model_name(&self) -> String {
        let model = Self::E::model_name();
        let col = self.as_str();

        let len = model.len() + 1 + col.len();
        let mut s = String::with_capacity(len);

        s.push_str(model);
        s.push('.');
        s.push_str(col);

        s
    }

    /// Build a data loader cache key from model, column, requested look ahead fields and suffix,
    /// so calls that select different field sets do not collide in the loader cache.
    fn to_loader_key(&self, look_ahead: &[LookaheadX<Self::E>], suffix: &str) -> String {
        let model = Self::E::model_name();
        let col = self.as_str();

        let len = model.len()
            + 1
            + col.len()
            + 1
            + look_ahead.iter().map(|l| l.c.len() + 1).sum::<usize>()
            + 1
            + suffix.len();
        let mut s = String::with_capacity(len);

        s.push_str(model);
        s.push('.');
        s.push_str(col);
        s.push('-');

        for l in look_ahead {
            s.push_str(l.c);
            s.push(',');
        }

        s.push('-');
        s.push_str(suffix);

        s
    }
}
