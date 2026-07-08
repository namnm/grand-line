use super::prelude::*;

/// Helper trait to abstract extra methods into `sea_orm` column.
pub trait ColumnX<E>
where
    E: EntityX,
    Self: ColumnTrait,
{
    fn to_string_with_model_name(&self) -> String {
        let model = E::model_name();
        let col = self.as_str();

        let len = model.len() + 1 + col.len();
        let mut s = String::with_capacity(len);

        s.push_str(model);
        s.push('.');
        s.push_str(col);

        s
    }

    fn to_loader_key(&self, look_ahead: &[LookaheadX<E>], exclude_deleted: bool) -> String {
        let exclude_deleted = if exclude_deleted {
            "exclude_deleted-"
        } else {
            ""
        };
        let model = E::model_name();
        let col = self.as_str();

        let len = model.len()
            + 1
            + col.len()
            + 1
            + exclude_deleted.len()
            + look_ahead.iter().map(|l| l.c.len() + 1).sum::<usize>();
        let mut s = String::with_capacity(len);

        s.push_str(model);
        s.push('.');
        s.push_str(col);
        s.push('-');
        s.push_str(exclude_deleted);

        let mut first = true;
        for l in look_ahead {
            if !first {
                s.push(',');
            }
            first = false;
            s.push_str(l.c);
        }

        s
    }
}
