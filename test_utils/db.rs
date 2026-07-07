use crate::prelude::*;
use core::time::Duration;

// ---------------------------------------------------------------------------
// Entry point -- create a fresh temporary database for the active backend
// ---------------------------------------------------------------------------

/// Create a temporary database using the URI matching the compiled backend feature.
pub async fn tmp_db() -> Res<TmpDb> {
    let tmp = TmpDb::new(db_uri()).await?;
    Ok(tmp)
}
const fn db_uri() -> &'static str {
    if cfg!(feature = "postgres") {
        return "postgres://postgres:test_pwd@localhost:5432/test_db";
    }
    if cfg!(feature = "mysql") {
        return "mysql://root:test_pwd@localhost:3306/test_db";
    }
    "sqlite::memory:"
    // already compile_error! in lib.rs to check postgres, mysql, sqlite
}

// ============================================================================
// create temporary db and automatically clean up on drop
// ============================================================================

/// A temporary, isolated database for a single test, call drop() when done.
pub struct TmpDb {
    ty: TmpDbTy,
    name: String,
    pub db: DatabaseConnection,
}

enum TmpDbTy {
    Postgres,
    MySql {
        admin: DatabaseConnection,
    },
    Sqlite,
}

impl TmpDb {
    /// Create an isolated Postgres schema, MySQL database, or in-memory
    /// Sqlite database, based on the scheme of uri.
    pub async fn new(uri: &str) -> Res<Self> {
        match get_uri_scheme(uri).as_str() {
            "postgres" => Self::new_postgres(uri).await,
            "mysql" => Self::new_mysql(uri).await,
            "sqlite" => Self::new_sqlite(uri).await,
            #[allow(clippy::panic)]
            _ => panic!("TmpDb::new expect postgres|mysql|sqlite: {uri}"),
        }
    }

    async fn new_postgres(uri: &str) -> Res<Self> {
        let name = new_db_name();
        let db = conn(uri).await?;

        let stmt = format!("CREATE SCHEMA {name};");
        exec(&db, DbBackend::Postgres, &stmt).await?;

        let stmt = format!("SET search_path TO {name};");
        exec(&db, DbBackend::Postgres, &stmt).await?;

        Ok(Self {
            name,
            db,
            ty: TmpDbTy::Postgres,
        })
    }

    async fn new_mysql(uri: &str) -> Res<Self> {
        let name = new_db_name();
        let admin = conn(uri).await?;

        let stmt = format!("CREATE DATABASE {name};");
        exec(&admin, DbBackend::MySql, &stmt).await?;

        let uri = replace_db_name(uri, &name);
        let db = conn(&uri).await?;

        Ok(Self {
            name,
            db,
            ty: TmpDbTy::MySql {
                admin,
            },
        })
    }

    async fn new_sqlite(_uri: &str) -> Res<Self> {
        let name = "memory".to_owned();
        let db = conn("sqlite::memory:").await?;

        Ok(Self {
            name,
            db,
            ty: TmpDbTy::Sqlite,
        })
    }

    /// Tear down the temporary schema/database and close the connection,
    /// must be called explicitly, this is not the std Drop trait.
    pub async fn drop(&self) -> Res<()> {
        match &self.ty {
            TmpDbTy::Postgres => {
                let stmt = "SET search_path TO public;";
                exec(&self.db, DbBackend::Postgres, stmt).await?;
                let name = &self.name;
                let stmt = format!("DROP SCHEMA IF EXISTS {name} CASCADE;");
                exec(&self.db, DbBackend::Postgres, &stmt).await?;
                self.db.clone().close().await?;
            }
            TmpDbTy::MySql {
                admin,
            } => {
                self.db.clone().close().await?;
                let name = &self.name;
                let stmt = format!("DROP DATABASE IF EXISTS {name};");
                exec(admin, DbBackend::MySql, &stmt).await?;
            }
            TmpDbTy::Sqlite => {
                self.db.clone().close().await?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// helpers
// ============================================================================

fn new_db_name() -> String {
    let id = ulid();
    format!("test_{id}")
}

fn get_uri_scheme(uri: &str) -> String {
    uri.split(':').next().unwrap_or_default().to_owned()
}

fn replace_db_name(uri: &str, name: &str) -> String {
    if let Some((head, _)) = uri.rsplit_once('/') {
        format!("{head}/{name}")
    } else {
        let head = uri.trim_end_matches('/');
        format!("{head}/{name}")
    }
}

async fn conn(uri: &str) -> Res<DatabaseConnection> {
    let mut c = ConnectOptions::new(uri);
    let d = Duration::from_secs(1);
    c.connect_timeout(d);
    c.acquire_timeout(d);

    // prevent search_path override in migration
    #[cfg(feature = "postgres")]
    c.max_connections(1);

    let db = Database::connect(c).await?;

    Ok(db)
}

async fn exec(db: &DatabaseConnection, be: DbBackend, stmt: &str) -> Res<()> {
    let stmt = Statement::from_string(be, stmt);
    db.execute(stmt).await?;

    Ok(())
}

/// Create the table for an entity in the given database, using its schema.
pub async fn create_table<E>(db: &DatabaseConnection, e: E) -> Res<()>
where
    E: EntityX,
{
    let backend = db.get_database_backend();
    let schema = DbSchema::new(backend);
    let stmt = schema.create_table_from_entity(e);
    db.execute(backend.build(&stmt)).await?;

    Ok(())
}
