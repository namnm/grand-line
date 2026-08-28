use super::prelude::*;

/// Helper trait to abstract extra methods into sea_orm entity.
#[async_trait]
pub trait EntityX
where
    Self: EntityTrait<Model = Self::M, ActiveModel = Self::A, Column = Self::C> + Send + Sync,
{
    // ---------------------------------------------------------------------------
    // Associated types and entity metadata
    // ---------------------------------------------------------------------------

    /// Sea_orm model type for this entity.
    type M: ModelX<Self>;
    /// Sea_orm active model type for this entity.
    type A: ActiveModelX<Self>;
    /// Column enum type for this entity.
    type C: ColumnX<E = Self>;
    /// Graphql filter input type for this entity.
    type F: FilterX<E = Self>;
    /// Graphql order_by enum type for this entity.
    type O: OrderBy<E = Self>;
    /// Graphql output model type for this entity.
    type G: GqlModel<Self>;

    /// Get entity model name.
    /// To clarify model name in case of error.
    fn model_name() -> &'static str;

    // ---------------------------------------------------------------------------
    // Column accessors
    // ---------------------------------------------------------------------------

    /// Get column id.
    /// Should be generated in the model macro.
    fn col_id() -> Self::C;
    /// Get column created_at.
    /// Should be generated in the model macro.
    fn col_created_at() -> Option<Self::C>;
    /// Get column updated_at.
    /// Should be generated in the model macro.
    fn col_updated_at() -> Option<Self::C>;
    /// Get column deleted_at.
    /// Should be generated in the model macro.
    fn col_deleted_at() -> Option<Self::C>;
    /// Get column created_by_id.
    /// Should be generated in the model macro.
    fn col_created_by_id() -> Option<Self::C>;
    /// Get column updated_by_id.
    /// Should be generated in the model macro.
    fn col_updated_by_id() -> Option<Self::C>;
    /// Get column deleted_by_id.
    /// Should be generated in the model macro.
    fn col_deleted_by_id() -> Option<Self::C>;

    // ---------------------------------------------------------------------------
    // GraphQL field lookahead metadata
    // ---------------------------------------------------------------------------

    /// Get sql columns map with rust snake field name, for gql look ahead.
    /// Holds every column, #[graphql(skip)] ones included, they are still needed
    /// as an sql_dep. Use gql_select as the source of what is reachable over gql.
    /// Should be generated in the model macro.
    fn gql_cols() -> &'static LazyLock<HashMap<&'static str, Self::C>>;
    /// Get sql exprs map with rust snake field name, for gql look ahead.
    /// Should be generated in the model macro.
    fn gql_exprs() -> &'static LazyLock<HashMap<&'static str, SimpleExpr>>;
    /// Get rust snake field name sql columns, from gql camel field, for gql look ahead.
    /// To look ahead and select only requested fields in the gql context.
    /// Should be generated in the model macro.
    fn gql_select() -> &'static LazyLock<HashMap<&'static str, HashSet<&'static str>>>;

    /// Returns true when this entity enables history tracking.
    /// Should be generated in the model macro.
    fn has_history() -> bool;

    /// Rust field names dropped from the History data snapshot,
    /// every #[graphql(skip)] plus every #[history(skip)] column.
    /// Should be generated in the model macro.
    fn history_skip() -> &'static [&'static str];

    /// Look ahead for sql columns and exprs, from requested fields in the gql context.
    /// Only meaningful at a position whose own selection set names this entity's
    /// fields, i.e. a relation field, not a scalar one.
    fn gql_look_ahead(ctx: &Context<'_>) -> Res<Vec<LookaheadX<Self>>> {
        let f = ctx.look_ahead().selection_fields();
        let f = f.first().ok_or(MyErr::GqlLookAhead)?;
        Ok(Self::gql_look_ahead_of(f.selection_set()))
    }

    /// Same as gql_look_ahead but reading the selection set of the named child
    /// field, for payload wrappers that nest the row one level down, e.g. the
    /// node field of a subscription event. Empty when the child was not selected.
    fn gql_look_ahead_at(ctx: &Context<'_>, field: &str) -> Res<Vec<LookaheadX<Self>>> {
        let f = ctx.look_ahead().selection_fields();
        let f = f.first().ok_or(MyErr::GqlLookAhead)?;
        let Some(child) = f.selection_set().find(|c| c.name() == field) else {
            return Ok(vec![]);
        };
        Ok(Self::gql_look_ahead_of(child.selection_set()))
    }

    /// Maps one selection set to the sql columns and exprs backing its fields.
    fn gql_look_ahead_of<'a, I>(fields: I) -> Vec<LookaheadX<Self>>
    where
        I: Iterator<Item = SelectionField<'a>>,
    {
        let gql_select = Self::gql_select();
        Self::gql_look_ahead_of_names(
            fields
                .filter_map(|f| gql_select.get(f.name().to_owned().as_str()))
                .flat_map(|c| c.iter().copied()),
        )
    }

    /// Every field reachable over gql, for a caller whose own context does not
    /// describe a selection of this entity, e.g. a scalar #[resolver] loading a
    /// related row. Built from gql_select, not gql_cols, so #[graphql(skip)]
    /// columns stay out. Evaluates every #[sql_expr] the entity has, name the
    /// columns with gql_look_ahead_cols when that cost matters.
    fn gql_look_ahead_all() -> Vec<LookaheadX<Self>> {
        Self::gql_look_ahead_of_names(Self::gql_select().values().flat_map(|c| c.iter().copied()))
    }

    /// Exactly these columns, for a caller that knows what it needs.
    /// Covers plain columns only, an #[sql_expr] field has no Column to name.
    fn gql_look_ahead_cols(cols: &[Self::C]) -> Vec<LookaheadX<Self>> {
        Self::gql_look_ahead_of_names(cols.iter().map(|c| c.as_str()))
    }

    /// Maps rust snake field names to the sql columns and exprs backing them.
    /// BTreeSet rather than HashSet on purpose: ColumnX::to_loader_key writes this
    /// order into the data loader cache key, and every upstream source here is a
    /// HashMap or HashSet, whose iteration order differs per instance. An unstable
    /// order splits one batch into a loader per row, which is the n+1 the loader exists
    /// to prevent. Ordering by content, not by insertion, is what makes it stable.
    fn gql_look_ahead_of_names<I>(names: I) -> Vec<LookaheadX<Self>>
    where
        I: Iterator<Item = &'static str>,
    {
        let gql_cols = Self::gql_cols();
        let gql_exprs = Self::gql_exprs();

        names
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter_map(|c| {
                let col = gql_cols.get(c);
                let expr = gql_exprs.get(c);
                match (col, expr) {
                    (None, None) => None,
                    _ => Some(LookaheadX {
                        c,
                        col: col.copied(),
                        expr: expr.cloned(),
                    }),
                }
            })
            .collect::<Vec<_>>()
    }

    // ---------------------------------------------------------------------------
    // Condition and soft-delete builders
    // ---------------------------------------------------------------------------

    /// Quickly build condition id eq.
    fn cond_id(id: &str) -> Condition {
        Condition::all().add(Self::col_id().eq(id))
    }

    /// Ensure deleted_at column is present.
    fn ensure_col_deleted_at() -> Res<Self::C> {
        let col = Self::col_deleted_at().ok_or_else(|| MyErr::DbCol404 {
            col: Self::model_name().to_owned() + ".deleted_at",
        })?;
        Ok(col)
    }
    /// Quickly build condition exclude deleted.
    fn cond_exclude_deleted() -> Option<Condition> {
        Self::col_deleted_at().map(|c| Condition::all().add(c.is_null()))
    }

    /// Set deleted_at with filter by id.
    /// It also checks if the model has configured with deleted_at column or not.
    fn soft_delete_by_id(id: &str) -> Res<UpdateMany<Self>> {
        let r = Self::soft_delete_many()?.filter_by_id(id);
        Ok(r)
    }

    /// Set deleted_at without any filter.
    /// It also checks if the model has configured with deleted_at column or not.
    fn soft_delete_many() -> Res<UpdateMany<Self>> {
        Self::ensure_col_deleted_at()?;
        let am = Self::A::defaults_on_delete();
        let r = Self::update_many().set(am);
        Ok(r)
    }

    // ---------------------------------------------------------------------------
    // GraphQL resolver helpers
    // ---------------------------------------------------------------------------

    /// Helper to use in resolver body of the macro search.
    async fn gql_search<D>(
        ctx: &Context<'_>,
        tx: &D,
        // From graphql input.
        filter: Option<Self::F>,
        order_by: Option<Vec<Self::O>>,
        page: Option<Pagination>,
        include_deleted: Option<bool>,
        // From resolver handler, relation, and authz row filter.
        extra: Search<Self::O>,
    ) -> Res<Vec<Self::G>>
    where
        D: ConnectionTrait,
    {
        let c = ctx.core_config();
        let r = Self::find()
            .include_deleted(extra.include_deleted(include_deleted, filter.as_ref()))
            .filter(extra.filter.condition)
            .filter_option(filter)
            .chain(order_by.combine(extra.default_order_by, c))
            .chain(page.inner(c))
            .gql_select(ctx)?
            .all(tx)
            .await?;
        Ok(r)
    }

    /// Helper to use in resolver body of the macro count.
    async fn gql_count<D>(
        _ctx: &Context<'_>,
        tx: &D,
        // From graphql input.
        filter: Option<Self::F>,
        include_deleted: Option<bool>,
        // From resolver handler and authz row filter.
        extra: Count,
    ) -> Res<u64>
    where
        D: ConnectionTrait,
    {
        let r = Self::find()
            .include_deleted(extra.include_deleted(include_deleted, filter.as_ref()))
            .filter(extra.condition)
            .filter_option(filter)
            .count(tx)
            .await?;
        Ok(r)
    }

    /// Helper to use in resolver body of the macro detail.
    async fn gql_detail<D>(
        ctx: &Context<'_>,
        tx: &D,
        // From graphql input.
        id: &str,
        include_deleted: Option<bool>,
        // From resolver handler and authz row filter.
        extra: Detail,
    ) -> Res<Option<Self::G>>
    where
        D: ConnectionTrait,
    {
        let r = Self::find()
            .include_deleted(extra.include_deleted(include_deleted, Option::<&Self::F>::None))
            .filter_by_id(id)
            .filter(extra.condition)
            .gql_select(ctx)?
            .one(tx)
            .await?;
        Ok(r)
    }

    /// Helper to use in resolver body of the macro update/delete.
    async fn gql_mutation_check_id<D>(
        _ctx: &Context<'_>,
        tx: &D,
        id: &str,
        authz_row: Option<Self::F>,
        authz_err: &GrandLineErr,
    ) -> Res<()>
    where
        D: ConnectionTrait,
    {
        let q = || Self::find().filter_by_id(id);

        let Some(f) = authz_row else {
            q().exists_or_404(tx).await?;
            return Ok(());
        };

        if !q().filter(f).exists(tx).await? {
            return Err(authz_err.clone());
        }

        Ok(())
    }

    /// Helper to use in resolver body of the macro update.
    async fn gql_update<D>(
        ctx: &Context<'_>,
        tx: &D,
        id: &str,
        am: Self::A,
        authz_row: Option<Self::F>,
        authz_err: &GrandLineErr,
    ) -> Res<Self::G>
    where
        D: ConnectionTrait,
    {
        let rows_affected = Self::update_many()
            .filter_by_id(id)
            .filter_option(authz_row)
            .set(am)
            .exec(tx)
            .await?
            .rows_affected;

        if rows_affected == 0 {
            return Err(authz_err.clone());
        }

        let r = Self::find().filter_by_id(id).gql_select(ctx)?.one_or_404(tx).await?;
        Ok(r)
    }

    /// Helper to use in resolver body of the macro delete.
    /// by_id is the current user id for history tracking (None if not authenticated).
    async fn gql_delete<D>(
        _ctx: &Context<'_>,
        tx: &D,
        id: &str,
        permanent: Option<bool>,
        authz_row: Option<Self::F>,
        authz_err: &GrandLineErr,
        by_id: Option<String>,
    ) -> Res<Self::G>
    where
        D: ConnectionTrait,
    {
        let rows_affected = if permanent.unwrap_or_default() {
            if Self::has_history()
                && let Some(m) = Self::find().filter_by_id(id).one(tx).await?
            {
                History::add(tx, HistoryOperation::PermanentDelete, &m, by_id).await?;
            }
            Self::delete_many()
                .filter_by_id(id)
                .filter_option(authz_row)
                .exec(tx)
                .await?
                .rows_affected
        } else {
            let rows = Self::soft_delete_by_id(id)?
                .filter_option(authz_row)
                .exec(tx)
                .await?
                .rows_affected;
            if Self::has_history()
                && rows > 0
                && let Some(m) = Self::find().filter_by_id(id).one(tx).await?
            {
                History::add(tx, HistoryOperation::Delete, &m, by_id).await?;
            }
            rows
        };

        if rows_affected == 0 {
            return Err(authz_err.clone());
        }

        let r = Self::G::from_id(id);
        Ok(r)
    }

    /// Helper to use in resolver body of has_one / belongs_to.
    /// extra is the relation's own resolver-contributed filter (if any), folded into
    /// the condition and the batch key the same way authz_row already is - both are
    /// Self::F so both are serializable, keeping different (authz_row, extra) pairs
    /// from colliding into the same dataloader batch.
    ///
    /// Loads the columns named by the calling field's own gql selection set, so it
    /// may only be called from a field whose selection set describes this entity,
    /// i.e. a relation field. From a scalar #[resolver] there is no such selection
    /// set and this errors, use gql_load_with there.
    async fn gql_load<D>(
        ctx: &Context<'_>,
        tx: &D,
        col: Self::C,
        id: String,
        authz_row: Option<Self::F>,
        include_deleted: Option<bool>,
        extra: Option<Self::F>,
    ) -> Res<Option<Self::G>>
    where
        D: ConnectionTrait,
    {
        let f = ctx.look_ahead().selection_fields();
        let f = f.first().ok_or(MyErr::GqlLookAhead)?;
        // An empty selection set means a scalar field, gql requires one on anything
        // returning an object type. So this position cannot be describing Self, and
        // every column would come back None. A selection set that simply names no
        // column of Self is a different thing and stays valid, e.g. __typename only,
        // or a #[resolver] field declaring no sql_dep.
        if f.selection_set().next().is_none() {
            return Err(MyErr::GqlLoadNoSelectionSet {
                model: Self::model_name().to_owned(),
                field: ctx.field().name().to_owned(),
            }
            .into());
        }

        let look_ahead = Self::gql_look_ahead_of(f.selection_set());
        Self::gql_load_with(ctx, tx, col, id, authz_row, include_deleted, extra, look_ahead).await
    }

    /// Same as gql_load with the loaded columns given explicitly, for a caller whose
    /// own gql position does not describe this entity. Pair it with
    /// gql_look_ahead_cols to name the columns, or gql_look_ahead_all for the whole row.
    #[allow(clippy::too_many_arguments)]
    async fn gql_load_with<D>(
        ctx: &Context<'_>,
        _tx: &D,
        col: Self::C,
        id: String,
        authz_row: Option<Self::F>,
        include_deleted: Option<bool>,
        extra: Option<Self::F>,
        look_ahead: Vec<LookaheadX<Self>>,
    ) -> Res<Option<Self::G>>
    where
        D: ConnectionTrait,
    {
        let cond_exclude_deleted = if include_deleted.unwrap_or_default() {
            None
        } else {
            Self::cond_exclude_deleted()
        };

        let deleted_marker = if cond_exclude_deleted.is_none() {
            "include_deleted"
        } else {
            "exclude_deleted"
        };
        // Serialize authz_row and extra together as one JSON value rather than
        // concatenating their individual json strings - each can contain arbitrary
        // characters (dates, free-form text, etc.), so naive string-joining them
        // could let two different (authz_row, extra) pairs collide on the same key.
        let suffix = json_string(&(deleted_marker, authz_row.as_ref(), extra.as_ref()))?;

        let key = col.to_loader_key(&look_ahead, &suffix);

        let condition = Condition::all()
            .add_option(authz_row.map(|f| f.into_condition()))
            .add_option(extra.map(|f| f.into_condition()))
            .add_option(cond_exclude_deleted);

        ctx.data_loader(key, col, look_ahead, condition)
            .await?
            .as_ref()
            .load_one(id)
            .await
    }
}
