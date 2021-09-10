use futures::prelude::*;
use futures::{FutureExt, TryFutureExt};
use gotham::handler::{HandlerFuture, HandlerResult};
use gotham::middleware::{Middleware, NewMiddleware};
use gotham::state::State;
use logger::*;
use sea_orm::DatabaseConnection;
use sqlx::{
    any::AnyRow, mysql::MySqlRow, postgres::PgRow, sqlite::SqliteRow, MySqlPool, PgPool, SqlitePool,
};
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::pin::Pin;
use std::sync::Arc;

#[derive(Clone)]
pub enum FydiaPool {
    Mysql(MySqlPool),
    PgSql(PgPool),
    Sqlite(SqlitePool),
}

pub trait ToAnyRows {
    fn to_anyrows(self) -> Vec<AnyRow>;
}

impl ToAnyRows for Vec<MySqlRow> {
    fn to_anyrows(self) -> Vec<AnyRow> {
        let mut result = Vec::new();
        for i in self {
            result.push(AnyRow::from(i));
        }
        result
    }
}

impl ToAnyRows for Vec<PgRow> {
    fn to_anyrows(self) -> Vec<AnyRow> {
        let mut result = Vec::new();
        for i in self {
            result.push(AnyRow::from(i));
        }
        result
    }
}

impl ToAnyRows for Vec<SqliteRow> {
    fn to_anyrows(self) -> Vec<AnyRow> {
        let mut result = Vec::new();
        for i in self {
            result.push(AnyRow::from(i));
        }
        result
    }
}

pub trait ToAnyRow {
    fn to_anyrow(self) -> AnyRow;
}

impl ToAnyRow for MySqlRow {
    fn to_anyrow(self) -> AnyRow {
        AnyRow::from(self)
    }
}

impl ToAnyRow for PgRow {
    fn to_anyrow(self) -> AnyRow {
        AnyRow::from(self)
    }
}

impl ToAnyRow for SqliteRow {
    fn to_anyrow(self) -> AnyRow {
        AnyRow::from(self)
    }
}

pub fn parse_array(parse: String) -> Vec<String> {
    let mut result = Vec::new();

    let split = if let Some(prefix_stripped) = parse.strip_prefix('[') {
        if let Some(suffix_stripped) = prefix_stripped.strip_suffix(']') {
            suffix_stripped.split(',')
        } else {
            return Vec::new();
        }
    } else {
        return Vec::new();
    };

    for i in split {
        let striped = i.replace('"', "").replace(" ", "");

        result.push(striped.to_string())
    }

    result
}

#[derive(StateData, Clone)]
pub struct Repo {
    pool: Arc<DatabaseConnection>,
}

impl Repo {
    pub fn new(pool: DatabaseConnection) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }

    pub fn new_arc(pool: Arc<DatabaseConnection>) -> Self {
        Self { pool }
    }

    pub fn get_pool(&self) -> Arc<DatabaseConnection> {
        self.pool.clone()
    }
}

#[derive(StateData)]
pub struct SqlPool {
    repo: AssertUnwindSafe<Repo>,
}

impl SqlPool {
    pub fn get_pool(&self) -> Arc<DatabaseConnection> {
        self.repo.get_pool()
    }
}

impl Clone for SqlPool {
    fn clone(&self) -> Self {
        match catch_unwind(|| self.repo.get_pool()) {
            Ok(repo) => SqlPool {
                repo: AssertUnwindSafe(Repo::new_arc(repo)),
            },
            Err(_) => {
                error!("PANIC: clone caused a panic".to_string());
                std::process::abort();
            }
        }
    }
}

impl SqlPool {
    pub fn new(repo: Repo) -> Self {
        Self {
            repo: AssertUnwindSafe(repo),
        }
    }
}

impl NewMiddleware for SqlPool {
    type Instance = SqlPool;

    fn new_middleware(&self) -> gotham::anyhow::Result<Self::Instance> {
        match catch_unwind(|| self.repo.get_pool()) {
            Ok(e) => Ok(SqlPool {
                repo: AssertUnwindSafe(Repo::new_arc(e)),
            }),
            Err(_) => {
                error!("Error new middleware");
                std::process::abort()
            }
        }
    }
}

impl Middleware for SqlPool {
    fn call<Chain>(
        self,
        mut state: State,
        chain: Chain,
    ) -> Pin<Box<dyn Future<Output = HandlerResult> + Send>>
    where
        Chain: FnOnce(State) -> Pin<Box<HandlerFuture>> + Send + 'static,
        Self: Sized,
    {
        info!(format!("[{}] pre chain", gotham::state::request_id(&state)));
        state.put(SqlPool::new(Repo::new_arc(self.repo.get_pool())));

        let f = chain(state).and_then(move |(state, response)| {
            {
                info!(format!(
                    "[{}] post chain",
                    gotham::state::request_id(&state)
                ));
            }
            future::ok((state, response))
        });
        f.boxed()
    }
}
