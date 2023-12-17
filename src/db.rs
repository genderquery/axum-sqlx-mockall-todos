use async_trait::async_trait;
use sqlx::{query_as, Pool, Sqlite, SqlitePool};

use crate::{
    provider::{ProviderError, TodoProvider},
    Todo,
};

#[derive(Clone)]
pub struct SqliteTodoProvider {
    pool: SqlitePool,
}

impl From<&Pool<Sqlite>> for SqliteTodoProvider {
    fn from(value: &Pool<Sqlite>) -> Self {
        SqliteTodoProvider {
            pool: value.clone(),
        }
    }
}

#[async_trait]
impl TodoProvider for SqliteTodoProvider {
    async fn get_todos(&self) -> Result<Vec<Todo>, ProviderError> {
        let todos = query_as!(Todo, "select * from todos")
            .fetch_all(&self.pool)
            .await?;
        Ok(todos)
    }

    async fn get_todo(&self, id: i64) -> Result<Option<Todo>, ProviderError> {
        let todo = query_as!(Todo, "select * from todos where id=?1", id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(todo)
    }

    async fn add_todo(&self, description: &str) -> Result<Todo, ProviderError> {
        let todo = query_as!(
            Todo,
            "insert into todos (description) values (?1) returning *",
            description
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(todo)
    }

    async fn update_todo(
        &self,
        id: i64,
        description: &str,
        done: bool,
    ) -> Result<Todo, ProviderError> {
        let todo = query_as!(
            Todo,
            // Work-around for bug where id gets returned as nullable
            "update todos set description=?1, done=?2 where id=?3
            returning id as \"id!\", description, done",
            description,
            done,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(todo)
    }
}

impl From<sqlx::Error> for ProviderError {
    fn from(value: sqlx::Error) -> Self {
        ProviderError(value.into())
    }
}
