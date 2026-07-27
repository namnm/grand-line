mod auth;
mod authz;
mod models;
mod utils;

pub mod prelude {
    pub use crate::auth::*;
    pub use crate::authz::*;
    pub use crate::models::*;
    pub use crate::utils::*;
    pub use grand_line::prelude::*;
}

use crate::prelude::*;

// ----------------------------------------------------------------------------
// init schema
// ----------------------------------------------------------------------------

grand_line::include_generated_schema! {}

/// The app's fully-wired GraphQL schema type, no subscriptions.
pub type AppSchema = GraphQLSchema<Query, Mutation, EmptySubscription>;

/// Builds the app schema wired with db and the real Saas auth/authz implementations.
pub fn schema(db: &DatabaseConnection) -> SchemaBuilder<Query, Mutation, EmptySubscription> {
    let session_impl: Box<dyn AuthSessionImpl> = Box::new(SaasAuthSessionImpl);
    let otp_impl: Box<dyn AuthOtpImpl> = Box::new(SaasAuthOtpImpl);
    let org_impl = Org::authz_default_impl();
    let role_impl: Box<dyn AuthzRoleImpl> = Box::new(SaasRoleImpl);

    GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
        .extension(GrandLineExtension)
        .data(Arc::new(db.clone()))
        .data(session_impl)
        .data(otp_impl)
        .data(org_impl)
        .data(role_impl)
}

// ----------------------------------------------------------------------------
// init db
// ----------------------------------------------------------------------------

/// Connects the app's sqlite database, creates every table, and seeds bootstrap data.
pub async fn db() -> Result<DatabaseConnection, Box<dyn Error + Send + Sync>> {
    let db = Database::connect("sqlite:file:saas?mode=memory&cache=shared").await?;

    let backend = db.get_database_backend();
    let schema = DbSchema::new(backend);
    for stmt in [
        schema.create_table_from_entity(User),
        schema.create_table_from_entity(LoginSession),
        schema.create_table_from_entity(Otp),
        schema.create_table_from_entity(Org),
        schema.create_table_from_entity(Role),
        schema.create_table_from_entity(UserInRole),
        schema.create_table_from_entity(Impersonation),
    ] {
        db.execute(&stmt).await?;
    }

    seed(&db).await?;

    Ok(db)
}

/// Seeds the bootstrap system user, admin role, and a demo org.
pub async fn seed(db: &DatabaseConnection) -> Result<(), Box<dyn Error + Send + Sync>> {
    let system = am_create!(User {
        email: "system@example.com".to_owned(),
        password_hashed: rand_utils::password_hash("123123")?,
    })
    .exec_without_ctx(db)
    .await?;

    let wildcard = json!({
        "*": {
            "inputs": {
                "allow": true,
                "children": {
                    "**": {
                        "allow": true,
                        "children": null,
                    },
                },
            },
            "output": {
                "allow": true,
                "children": {
                    "**": {
                        "allow": true,
                        "children": null,
                    },
                },
            },
        },
    });
    let role = am_create!(Role {
        name: "System".to_owned(),
        realm: "system".to_owned(),
        col_policy: wildcard,
        row_policy: json!({}),
        org_id: None,
    })
    .exec_without_ctx(db)
    .await?;

    am_create!(UserInRole {
        user_id: system.id.clone(),
        role_id: role.id.clone(),
        org_id: None,
    })
    .exec_without_ctx(db)
    .await?;

    am_create!(Org {
        name: "Acme Inc".to_owned(),
    })
    .exec_without_ctx(db)
    .await?;

    Ok(())
}
