use grand_line::prelude::*;

// ---------------------------------------------------------------------------
// Basic row filter handlers
// ---------------------------------------------------------------------------

pub struct NoneHandler;
#[async_trait]
impl AuthzHandlers for NoneHandler {
    async fn execute_script(&self, _ctx: &Context<'_>, _script: &str) -> Res<Option<JsonValue>> {
        Ok(None)
    }
}

pub struct AssigneeHandler;
#[async_trait]
impl AuthzHandlers for AssigneeHandler {
    async fn execute_script(&self, ctx: &Context<'_>, _script: &str) -> Res<Option<JsonValue>> {
        let user_id = ctx.auth().await?;
        let f = json!({
            "assignee_id": user_id,
        });
        Ok(Some(f))
    }
}

pub struct OrgHandler;
#[async_trait]
impl AuthzHandlers for OrgHandler {
    async fn execute_script(&self, ctx: &Context<'_>, _script: &str) -> Res<Option<JsonValue>> {
        let org_id = ctx.authz().await?;
        let f = json!({
            "org_id": org_id,
        });
        Ok(Some(f))
    }
}

pub struct BothHandler;
#[async_trait]
impl AuthzHandlers for BothHandler {
    async fn execute_script(&self, ctx: &Context<'_>, _script: &str) -> Res<Option<JsonValue>> {
        let user_id = ctx.auth().await?;
        let org_id = ctx.authz().await?;
        let f = json!({
            "assignee_id": user_id,
            "org_id": org_id,
        });
        Ok(Some(f))
    }
}

// ---------------------------------------------------------------------------
// Script-value dependent handler
// ---------------------------------------------------------------------------

pub const SCRIPT_ALPHA: &str = "mock alpha script";
pub struct ScriptCheckHandler;
#[async_trait]
impl AuthzHandlers for ScriptCheckHandler {
    async fn execute_script(&self, _ctx: &Context<'_>, script: &str) -> Res<Option<JsonValue>> {
        let f = if script == SCRIPT_ALPHA {
            json!({
                "title": "Analyze the tissue sample",
            })
        } else {
            json!({
                "title": "Investigate the pattern",
            })
        };
        Ok(Some(f))
    }
}

// ---------------------------------------------------------------------------
// Script evaluation error handler
// ---------------------------------------------------------------------------

#[grand_line_err]
pub enum ScriptErr {
    #[error("evaluation failed")]
    Failed,
}

pub struct ErrorHandler;
#[async_trait]
impl AuthzHandlers for ErrorHandler {
    async fn execute_script(&self, _ctx: &Context<'_>, _script: &str) -> Res<Option<JsonValue>> {
        Err(ScriptErr::Failed.into())
    }
}

// ---------------------------------------------------------------------------
// Malformed filter response handlers
// ---------------------------------------------------------------------------

// Handler returning wrong JSON type: org_id expects String but receives a number.
// TaskFilter::from_json will fail deserialization -> InternalServer in GQL response.
pub struct WrongTypeHandler;
#[async_trait]
impl AuthzHandlers for WrongTypeHandler {
    async fn execute_script(&self, _ctx: &Context<'_>, _script: &str) -> Res<Option<JsonValue>> {
        let f = json!({
            "org_id": 123,
        });
        Ok(Some(f))
    }
}

// Handler returning an unknown field not present in TaskFilter.
// serde uses #[serde(default)] without deny_unknown_fields, so the field is
// silently dropped and the filter is effectively empty (no WHERE clause applied).
pub struct UnknownFieldHandler;
#[async_trait]
impl AuthzHandlers for UnknownFieldHandler {
    async fn execute_script(&self, _ctx: &Context<'_>, _script: &str) -> Res<Option<JsonValue>> {
        let f = json!({
            "unknown_col": "x",
        });
        Ok(Some(f))
    }
}
