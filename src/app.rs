use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use crate::{
    endpoints,
    provider::{ProviderError, TodoProvider},
};

pub type DynTodoProvider = Arc<dyn TodoProvider + Send + Sync>;
pub type AppState = State<DynTodoProvider>;

pub fn router(provider: DynTodoProvider) -> Router {
    Router::new()
        .route(
            "/todos",
            get(endpoints::get_todos).post(endpoints::add_todo),
        )
        .route(
            "/todos/:id",
            get(endpoints::get_todo).put(endpoints::update_todo),
        )
        .with_state(provider)
}

pub enum AppError {
    NotFound,
    InternalServerError(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND.into_response(),
            AppError::InternalServerError(err) => {
                tracing::error!("{}", err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        AppError::InternalServerError(err.into())
    }
}

impl From<ProviderError> for AppError {
    fn from(value: ProviderError) -> Self {
        AppError::InternalServerError(value.0)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use mockall::predicate::eq;
    use serde_json::{json, Value};
    use tower::ServiceExt;

    use crate::{endpoints::Todo, provider::MockTodoProvider};

    use super::*;

    #[tokio::test]
    async fn test_get_todos() {
        let mut provider = MockTodoProvider::new();
        provider.expect_get_todos().times(1).returning(|| {
            Ok(vec![
                Todo {
                    id: 1,
                    description: "test 1".to_string(),
                    done: false,
                },
                Todo {
                    id: 2,
                    description: "test 2".to_string(),
                    done: true,
                },
            ])
        });

        let app = router(Arc::new(provider));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/todos")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json,
            json!([{
                "id": 1,
                "description": "test 1",
                "done": false
            },  {
                "id": 2,
                "description": "test 2",
                "done": true
            }])
        );
    }

    #[tokio::test]
    async fn test_get_todo() {
        let mut provider = MockTodoProvider::new();
        provider
            .expect_get_todo()
            .times(1)
            .with(eq(1))
            .returning(|_| {
                Ok(Some(Todo {
                    id: 1,
                    description: "test 1".to_string(),
                    done: false,
                }))
            });

        let app = router(Arc::new(provider));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/todos/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json,
            json!({
                "id": 1,
                "description": "test 1",
                "done": false
            })
        );
    }

    #[tokio::test]
    async fn test_not_found() {
        let mut provider = MockTodoProvider::new();
        provider
            .expect_get_todo()
            .times(1)
            .with(eq(1))
            .returning(|_| Ok(None));

        let app = router(Arc::new(provider));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/todos/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn test_add_todo() {
        let mut provider = MockTodoProvider::new();
        provider
            .expect_add_todo()
            .times(1)
            .with(eq("test 1"))
            .returning(|_| {
                Ok(Todo {
                    id: 1,
                    description: "test 1".to_string(),
                    done: false,
                })
            });

        let app = router(Arc::new(provider));
        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/todos")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "description": "test 1",
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json,
            json!({
                "id": 1,
                "description": "test 1",
                "done": false
            })
        );
    }

    #[tokio::test]
    async fn test_update_todo() {
        let mut provider = MockTodoProvider::new();

        provider
            .expect_update_todo()
            .times(1)
            .with(eq(1), eq("test 1"), eq(true))
            .returning(|_, _, _| {
                Ok(Todo {
                    id: 1,
                    description: "test 1".to_string(),
                    done: true,
                })
            });

        let app = router(Arc::new(provider));

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::PUT)
                    .uri("/todos/1")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "description": "test 1",
                            "done": true,
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json,
            json!({
                "id": 1,
                "description": "test 1",
                "done": true
            })
        );
    }
}
