use std::net::SocketAddr;

use axum::{
    body::Body,
    http::{HeaderName, HeaderValue},
};
use axum_sqlx_mockall_todos::{app, db::SqliteTodoProvider, SqliteAppState};
use http_body_util::BodyExt;
use hyper::{
    client::conn::http1::{handshake, SendRequest},
    header, Request, StatusCode,
};
use hyper_util::rt::TokioIo;
use serde_json::{json, Value};
use sqlx::{Pool, Sqlite};
use tokio::net::{TcpListener, TcpStream};

async fn spawn_server(pool: Pool<Sqlite>) -> SocketAddr {
    let provider = SqliteTodoProvider::from(&pool);
    let state = SqliteAppState { provider };

    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app::router(state)).await.unwrap();
    });

    address
}

async fn client(address: SocketAddr) -> SendRequest<Body> {
    let stream = TcpStream::connect(address).await.unwrap();
    let io = TokioIo::new(stream);

    let (sender, connection) = handshake::<_, Body>(io).await.unwrap();

    tokio::task::spawn(async move {
        if let Err(err) = connection.await {
            panic!("Connection failed: {:?}", err);
        }
    });

    sender
}

fn has_json_content_type((k, v): (&HeaderName, &HeaderValue)) -> bool {
    k == header::CONTENT_TYPE && v == mime::APPLICATION_JSON.as_ref()
}

#[sqlx::test(fixtures("todos"))]
async fn test_get_todos(pool: Pool<Sqlite>) {
    let address = spawn_server(pool).await;
    let mut client = client(address).await;

    let req = Request::builder()
        .uri(format!("http://{address}/todos"))
        .body(Body::empty())
        .unwrap();

    let res = client.send_request(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().iter().any(has_json_content_type));

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        body,
        json!([{
            "id": 1,
            "description": "test 1",
            "done": false
        }, {
            "id": 2,
            "description": "test 2",
            "done": false
        }, {
            "id": 3,
            "description": "test 3",
            "done": false
        }])
    );
}

#[sqlx::test(fixtures("todos"))]
async fn test_get_todo(pool: Pool<Sqlite>) {
    let address = spawn_server(pool).await;
    let mut client = client(address).await;

    let req = Request::builder()
        .uri(format!("http://{address}/todos/1"))
        .body(Body::empty())
        .unwrap();

    let res = client.send_request(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().iter().any(has_json_content_type));

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        body,
        json!({
            "id": 1,
            "description": "test 1",
            "done": false
        })
    );
}

#[sqlx::test]
async fn test_not_found(pool: Pool<Sqlite>) {
    let address = spawn_server(pool).await;
    let mut client = client(address).await;

    let req = Request::builder()
        .uri(format!("http://{address}/todos/100"))
        .body(Body::empty())
        .unwrap();

    let res = client.send_request(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn test_add_todo(pool: Pool<Sqlite>) {
    let address = spawn_server(pool).await;
    let mut client = client(address).await;

    let req = Request::builder()
        .method(http::Method::POST)
        .uri(format!("http://{address}/todos"))
        .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .body(Body::from(
            serde_json::to_vec(&json!({
                "description": "test 1"
            }))
            .unwrap(),
        ))
        .unwrap();

    let res = client.send_request(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    assert!(res.headers().iter().any(has_json_content_type));

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body["description"], "test 1");
    assert_eq!(body["done"], false);

    let id = body["id"].as_i64().unwrap();

    let req = Request::builder()
        .uri(format!("http://{address}/todos/{id}"))
        .body(Body::empty())
        .unwrap();

    let res = client.send_request(req).await.unwrap();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body["id"], 1);
    assert_eq!(body["description"], "test 1");
    assert_eq!(body["done"], false);
}

#[sqlx::test(fixtures("todos"))]
async fn test_update_todo(pool: Pool<Sqlite>) {
    let address = spawn_server(pool).await;
    let mut client = client(address).await;

    let req = Request::builder()
        .method(http::Method::PUT)
        .uri(format!("http://{address}/todos/1"))
        .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .body(Body::from(
            serde_json::to_vec(&json!({
                "description": "test 1",
                "done": true
            }))
            .unwrap(),
        ))
        .unwrap();

    let res = client.send_request(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().iter().any(has_json_content_type));

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body["description"], "test 1");
    assert_eq!(body["done"], true);

    let id = body["id"].as_i64().unwrap();

    let req = Request::builder()
        .uri(format!("http://{address}/todos/{id}"))
        .body(Body::empty())
        .unwrap();

    let res = client.send_request(req).await.unwrap();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body["id"], 1);
    assert_eq!(body["description"], "test 1");
    assert_eq!(body["done"], true);
}
