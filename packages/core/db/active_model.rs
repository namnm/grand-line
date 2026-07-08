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

pub struct ActiveModelWrapper<T, E, A> {
    am: A,
    _phantom: PhantomData<(T, E)>,
}

impl<T, E, A> ActiveModelWrapper<T, E, A> {
    pub const fn new(am: A) -> Self {
        Self {
            am,
            _phantom: PhantomData,
        }
    }
}

// ============================================================================
// IntoActiveModel impls for bulk operations (e.g. Entity::insert_many)

impl<E, A> IntoActiveModel<A> for ActiveModelWrapper<AmCreate, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    fn into_active_model(self) -> A {
        self.am.set_defaults_on_create()
    }
}

impl<E, A> IntoActiveModel<A> for ActiveModelWrapper<AmUpdate, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    fn into_active_model(self) -> A {
        self.am.set_defaults_on_update()
    }
}

impl<E, A> IntoActiveModel<A> for ActiveModelWrapper<AmSoftDelete, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    fn into_active_model(self) -> A {
        self.am.set_defaults_on_delete()
    }
}

// ============================================================================
// exec_without_ctx

#[async_trait]
pub trait AmExecWithoutCtx: Sized {
    type Model: Send;
    async fn exec_without_ctx<D>(self, db: &D) -> Res<Self::Model>
    where
        D: ConnectionTrait;
}

#[async_trait]
impl<E, A> AmExecWithoutCtx for ActiveModelWrapper<AmCreate, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E> + Send,
    E::M: Send,
{
    type Model = E::M;

    async fn exec_without_ctx<D>(self, db: &D) -> Res<Self::Model>
    where
        D: ConnectionTrait,
    {
        let r = self.into_active_model().insert(db).await?;
        Ok(r)
    }
}

#[async_trait]
impl<E, A> AmExecWithoutCtx for ActiveModelWrapper<AmUpdate, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E> + Send,
    E::M: Send,
{
    type Model = E::M;

    async fn exec_without_ctx<D>(self, db: &D) -> Res<Self::Model>
    where
        D: ConnectionTrait,
    {
        let r = self.into_active_model().update(db).await?;
        Ok(r)
    }
}

#[async_trait]
impl<E, A> AmExecWithoutCtx for ActiveModelWrapper<AmSoftDelete, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E> + Send,
    E::M: Send,
{
    type Model = E::M;

    async fn exec_without_ctx<D>(self, db: &D) -> Res<Self::Model>
    where
        D: ConnectionTrait,
    {
        E::ensure_col_deleted_at()?;
        let r = self.into_active_model().update(db).await?;
        Ok(r)
    }
}

// ============================================================================
// ActiveModelX trait

/// Abstract extra active model methods implementation.
#[async_trait]
pub trait ActiveModelX<E>
where
    E: EntityX<A = Self>,
    Self: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Default + Send + Sync,
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

    /// `sea_orm` `ActiveModel` hooks will not be called with `Entity::` or bulk methods.
    /// We need to have this method instead to get default values on create.
    /// This will be used together with the macro `grand_line::am_create`.
    fn set_defaults_on_create(mut self) -> Self {
        if !self.get_id().is_set() {
            self = self.set_id(&ulid());
        }
        if !self.get_created_at().is_set() && E::col_created_at().is_some() {
            self = self.set_created_at(now());
        }
        self = self.set_defaults();
        self
    }
    /// Shortcut for `Self::default().set_defaults_on_create()`
    fn defaults_on_create() -> Self {
        <Self as Default>::default().set_defaults_on_create()
    }

    /// `sea_orm` `ActiveModel` hooks will not be called with `Entity::` or bulk methods.
    /// We need to have this method instead to get default values on update.
    /// This will be used together with the macro `grand_line::am_update`.
    fn set_defaults_on_update(mut self) -> Self {
        if !self.get_updated_at().is_set() && E::col_updated_at().is_some() {
            // do not call now() if there is no column
            self = self.set_updated_at(Some(now()));
        }
        self
    }
    /// Shortcut for `Self::default().set_defaults_on_update()`
    fn defaults_on_update() -> Self {
        <Self as Default>::default().set_defaults_on_update()
    }

    /// `sea_orm` `ActiveModel` hooks will not be called with `Entity::` or bulk methods.
    /// We need to have this method instead to get default values on delete.
    /// This will be used together with the macro `grand_line::am_soft_delete`.
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
    /// Shortcut for `Self::default().set_defaults_on_delete()`
    fn defaults_on_delete() -> Self {
        <Self as Default>::default().set_defaults_on_delete()
    }

    /// Set `deleted_at` and update db.
    /// It also checks if the model has configured with `deleted_at` column or not.
    async fn soft_delete<D>(self, db: &D) -> Res<E::M>
    where
        D: ConnectionTrait,
    {
        E::ensure_col_deleted_at()?;
        let r = self.set_defaults_on_delete().update(db).await?;
        Ok(r)
    }
}
