use super::prelude::*;

// ============================================================================
// ActiveModelX trait

/// Abstract extra active model methods implementation.
#[async_trait]
pub trait ActiveModelX<E>
where
    E: EntityX<A = Self>,
    Self: ActiveModelTrait<Entity = E> + TryIntoModel<E::M> + ActiveModelBehavior + Default + Send + Sync,
{
    /// Set default values from macro default.
    /// Should be generated in the model macro.
    fn set_defaults(self) -> Self;

    fn get_id(&self) -> ActiveValue<String>;
    fn set_id(self, v: &str) -> Self;

    fn get_created_at(&self) -> ActiveValue<DateTimeUtc>;
    fn set_created_at(self, v: DateTimeUtc) -> Self;
    fn get_updated_at(&self) -> ActiveValue<Option<DateTimeUtc>>;
    fn set_updated_at(self, v: Option<DateTimeUtc>) -> Self;
    fn get_deleted_at(&self) -> ActiveValue<Option<DateTimeUtc>>;
    fn set_deleted_at(self, v: Option<DateTimeUtc>) -> Self;

    fn get_created_by_id(&self) -> ActiveValue<Option<String>>;
    fn set_created_by_id(self, v: Option<String>) -> Self;
    fn get_updated_by_id(&self) -> ActiveValue<Option<String>>;
    fn set_updated_by_id(self, v: Option<String>) -> Self;
    fn get_deleted_by_id(&self) -> ActiveValue<Option<String>>;
    fn set_deleted_by_id(self, v: Option<String>) -> Self;

    /// sea_orm ActiveModel hooks will not be called with Entity:: or bulk methods.
    /// We need to have this method instead to get default values on create.
    /// This will be used together with the macro am_create!.
    fn set_defaults_on_create(mut self) -> Self {
        if self.get_id().is_not_set() {
            self = self.set_id(&ulid());
        }
        if E::col_created_at().is_some() && self.get_created_at().is_not_set() {
            self = self.set_created_at(now());
        }
        self = self.set_defaults();
        self
    }
    /// Shortcut for Self::default().set_defaults_on_create()
    fn defaults_on_create() -> Self {
        <Self as Default>::default().set_defaults_on_create()
    }

    /// sea_orm ActiveModel hooks will not be called with Entity:: or bulk methods.
    /// We need to have this method instead to get default values on update.
    /// This will be used together with the macro am_update!.
    fn set_defaults_on_update(mut self) -> Self {
        if E::col_updated_at().is_some() && !self.get_updated_at().is_set() {
            // do not call now() if there is no column
            self = self.set_updated_at(Some(now()));
        }
        self
    }
    /// Shortcut for Self::default().set_defaults_on_update()
    fn defaults_on_update() -> Self {
        <Self as Default>::default().set_defaults_on_update()
    }

    /// sea_orm ActiveModel hooks will not be called with Entity:: or bulk methods.
    /// We need to have this method instead to get default values on delete.
    /// This will be used together with the macro am_soft_delete!.
    fn set_defaults_on_delete(mut self) -> Self {
        self = self.set_defaults_on_update();
        if let Set(Some(v)) = self.get_updated_at() {
            self = self.set_deleted_at(Some(v));
        } else if E::col_updated_at().is_some() || E::col_deleted_at().is_some() {
            // do not call now() if there is no column
            let now = now();
            self = self.set_updated_at(Some(now)).set_deleted_at(Some(now));
        }
        self
    }
    /// Shortcut for Self::default().set_defaults_on_delete()
    fn defaults_on_delete() -> Self {
        <Self as Default>::default().set_defaults_on_delete()
    }

    /// Set deleted_at and update tx.
    /// It also checks if the model has configured with deleted_at column or not.
    async fn soft_delete<D>(self, tx: &D) -> Res<E::M>
    where
        D: ConnectionTrait,
    {
        E::ensure_col_deleted_at()?;
        let r = self.set_defaults_on_delete().update(tx).await?;
        Ok(r)
    }

    /// Ensure id is set, update and soft delete need it to target the right row.
    /// Unchanged counts too, e.g. ..model.into_active_model() marks id Unchanged
    /// rather than Set, only NotSet means the caller never provided it at all.
    fn ensure_id_set(&self) -> Res<()> {
        if self.get_id().is_not_set() {
            Err(MyErr::IdNotSet.into())
        } else {
            Ok(())
        }
    }
}
