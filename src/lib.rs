use app::AppState;
use db::SqliteTodoProvider;

pub mod app;
pub mod db;
pub mod endpoints;
pub mod provider;

#[derive(Clone)]
pub struct SqliteAppState {
    pub provider: SqliteTodoProvider,
}

impl AppState for SqliteAppState {
    type P = SqliteTodoProvider;

    fn provider(&self) -> &Self::P {
        &self.provider
    }
}
