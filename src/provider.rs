use async_trait::async_trait;

use crate::endpoints::Todo;

#[mockall::automock]
#[async_trait]
pub trait TodoProvider {
    async fn get_todos(&self) -> Result<Vec<Todo>, ProviderError>;
    async fn get_todo(&self, id: i64) -> Result<Option<Todo>, ProviderError>;
    async fn add_todo(&self, description: &str) -> Result<Todo, ProviderError>;
    async fn update_todo(
        &self,
        id: i64,
        description: &str,
        done: bool,
    ) -> Result<Todo, ProviderError>;
}

pub struct ProviderError(pub anyhow::Error);
