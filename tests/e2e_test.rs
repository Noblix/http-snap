use http_snap::run;
use serde_json::json;
use std::path::PathBuf;
use uuid::Uuid;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

mod common;

#[tokio::test]
async fn send_get_with_no_body() {
    common::init_logger();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/no-body"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"hello": "world"})))
        .mount(&server)
        .await;

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
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/times"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "RFC-2822": "Tue, 25 Mar 2025 14:54:09 GMT",
            "ISO-8601-Basic": "20250325T144509Z",
            "ISO-8601-Extended": "2025-03-25T14:54:09Z",
            "12‑Hour-Format": "03/25/2025 02:54:09 PM",
        })))
        .mount(&server)
        .await;

    let mut path = PathBuf::new();
    path.push("tests/e2e_inputs/compare_timestamp_formats.http");
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
async fn compare_guid_formats() {
    common::init_logger();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dishes/favorite"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("correlation-id", Uuid::new_v4().to_string().as_str())
                .set_body_json(json!({
                    "id": Uuid::new_v4().to_string(),
                    "name": "Beef Wellington"
                })),
        )
        .mount(&server)
        .await;

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
