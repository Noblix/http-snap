use http_snap::run;
use serde_json::json;
use std::path::PathBuf;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn send_get_with_no_body() {
    common::init_logger();
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/no-body")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({"hello": "world"}).to_string())
        .create();

    let mut path = PathBuf::new();
    path.push("tests/e2e_inputs/send_get_with_no_body.http");
    let result = run(
        &path,
        &common::create_environment_variables(&server),
        false,
        true,
    )
    .await
    .unwrap();

    assert_eq!(result, true);
}

#[tokio::test]
async fn compare_timestamp_formats() {
    common::init_logger();
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/times")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "RFC-2822": "Tue, 25 Mar 2025 14:54:09 GMT",
                "ISO-8601-Basic": "20250325T144509Z",
                "ISO-8601-Extended": "2025-03-25T14:54:09Z",
                "12‑Hour-Format": "03/25/2025 02:54:09 PM",
            })
            .to_string(),
        )
        .create();

    let mut path = PathBuf::new();
    path.push("tests/e2e_inputs/compare_timestamp_formats.http");
    let result = run(&path, &common::create_environment_variables(&server), false, true)
        .await
        .unwrap();

    assert_eq!(result, true);
}

#[tokio::test]
async fn compare_guid_formats() {
    common::init_logger();
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/dishes/favorite")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_header("correlation-id", &Uuid::new_v4().to_string())
        .with_body(
            json!({
                "id": &Uuid::new_v4().to_string(),
                "name": "Beef Wellington"
            })
            .to_string(),
        )
        .create();

    let mut path = PathBuf::new();
    path.push("tests/e2e_inputs/compare_guid_formats.http");
    let result = run(
        &path,
        &common::create_environment_variables(&server),
        false,
        true,
    )
    .await
    .unwrap();

    assert_eq!(result, true);
}
