use axum::{
    extract::{Path, State},
    Json,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    app::{AppError, AppState},
    provider::TodoProvider,
};

pub async fn get_todos<A: AppState>(State(state): State<A>) -> Result<Json<Vec<Todo>>, AppError> {
    let todos = state.provider().get_todos().await?;

    Ok(Json(todos))
}

pub async fn get_todo<A: AppState>(
    State(state): State<A>,
    Path(id): Path<i64>,
) -> Result<Json<Todo>, AppError> {
    let todo = state.provider().get_todo(id).await?;

    let todo = match todo {
        Some(todo) => todo,
        None => return Err(AppError::NotFound),
    };

    Ok(Json(todo))
}

pub async fn add_todo<A: AppState>(
    State(state): State<A>,
    Json(todo): Json<TodoAdd>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let TodoAdd { description } = todo;
    let todo = state.provider().add_todo(&description).await?;

    Ok((StatusCode::CREATED, Json(todo)))
}

pub async fn update_todo<A: AppState>(
    State(state): State<A>,
    Path(id): Path<i64>,
    Json(todo): Json<TodoUpdate>,
) -> Result<Json<Todo>, AppError> {
    let TodoUpdate { description, done } = todo;
    let todo = state.provider().update_todo(id, &description, done).await?;

    Ok(Json(todo))
}

#[derive(Serialize, Clone)]
pub struct Todo {
    pub id: i64,
    pub description: String,
    pub done: bool,
}

#[derive(Deserialize)]
pub struct TodoAdd {
    pub description: String,
}

#[derive(Deserialize)]
pub struct TodoUpdate {
    pub description: String,
    pub done: bool,
}
