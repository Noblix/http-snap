use http_snap::run;
use http_snap::types::{Detector, ExecuteOptions, UpdateMode};
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
        &ExecuteOptions::new_test(),
    )
    .await
    .unwrap();

    assert_eq!(result, true);
}

#[tokio::test]
async fn post_with_no_response() {
    common::init_logger();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/activate"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let mut path = PathBuf::new();
    path.push("tests/e2e_inputs/post_with_no_response.http");
    let result = run(
        &path,
        &common::create_environment_variables(&server),
        &ExecuteOptions::new_test(),
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
        &ExecuteOptions::new_test(),
    )
    .await
    .unwrap();

    assert_eq!(result, true);
}

#[tokio::test]
async fn detect_timestamp_formats() {
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
    path.push("tests/e2e_inputs/detect_timestamp_formats.http");
    let result = run(
        &path,
        &common::create_environment_variables(&server),
        &ExecuteOptions::new_update(true, UpdateMode::Overwrite, &[Detector::Timestamp]),
    )
    .await
    .unwrap();

    assert_eq!(result, true);
}

#[tokio::test]
async fn detect_timestamp_formats_keeping_values() {
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
    path.push("tests/e2e_inputs/detect_timestamp_formats_keeping_values.http");
    let result = run(
        &path,
        &common::create_environment_variables(&server),
        &ExecuteOptions::new_update(true, UpdateMode::Overwrite, &[Detector::Timestamp]),
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
        &ExecuteOptions::new_test(),
    )
    .await
    .unwrap();

    assert_eq!(result, true);
}

#[tokio::test]
async fn generate_guid() {
    common::init_logger();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/ids/.+$"))
        .respond_with(|req: &Request| {
            let id = req.url.path().rsplit('/').next().unwrap_or("unknown");
            ResponseTemplate::new(200).set_body_json(json!({
                "id": id,
                "name": "Echo"
            }))
        })
        .mount(&server)
        .await;

    let mut path = PathBuf::new();
    path.push("tests/e2e_inputs/generate_guid.http");
    let result = run(
        &path,
        &common::create_environment_variables(&server),
        &ExecuteOptions::new_test(),
    )
    .await
    .unwrap();

    assert_eq!(result, true);
}

#[tokio::test]
async fn detect_guid_and_timestamp() {
    common::init_logger();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex("/guid"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("correlation-id", Uuid::new_v4().to_string().as_str())
                .set_body_json(json!({
                    "id": Uuid::new_v4().to_string()
                })),
        )
        .mount(&server)
        .await;

    let mut path = PathBuf::new();
    path.push("tests/e2e_inputs/detect_guid_and_timestamp.http");
    let result = run(
        &path,
        &common::create_environment_variables(&server),
        &ExecuteOptions::new_update(
            true,
            UpdateMode::Overwrite,
            &[Detector::Guid, Detector::Timestamp],
        ),
    )
    .await
    .unwrap();

    assert_eq!(result, true);
}

#[tokio::test]
async fn import_and_use_variable() {
    common::init_logger();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tokens"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"token": "12345"})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/token/12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"found": true})))
        .mount(&server)
        .await;

    let mut path = PathBuf::new();
    path.push("tests/e2e_inputs/importer.http");
    let result = run(
        &path,
        &common::create_environment_variables(&server),
        &ExecuteOptions::new_test(),
    )
    .await
    .unwrap();

    assert_eq!(result, true);
}

#[tokio::test]
async fn writing_snapshot() {
    common::init_logger();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/complex"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
          "name": "Test Data",
          "id": 123,
          "active": true,
          "nestedObject": {
            "key1": "value1",
            "key2": {
              "subKey": 42,
              "subArray": [
                1,
                2,
                3,
                {
                  "deep": "value"
                }
              ]
            },
            "emptyObject": {}
          },
          "arrayOfObjects": [
            {
              "a": 1.0,
              "b": 2
            },
            {
              "a": 3,
              "b": 4
            }
          ],
          "emptyArray": [],
          "mixedArray": [
            null,
            "string",
            123,
            false,
            {}
          ],
          "deeplyNested": {
            "level1": {
              "level2": {
                "level3": {
                  "level4": "end"
                }
              }
            }
          },
          "specialChars": "Quotes \" and backslash \\ and newline \n end",
          "unicode": "こんにちは世界"
        })))
        .mount(&server)
        .await;

    let mut path = PathBuf::new();
    path.push("tests/e2e_inputs/writing_snapshot.http");
    let result = run(
        &path,
        &common::create_environment_variables(&server),
        &ExecuteOptions::new_update(true, UpdateMode::Overwrite, &[Detector::Timestamp]),
    )
    .await
    .unwrap();

    assert_eq!(result, true);
}

#[tokio::test]
async fn markdown_with_single_section() {
    common::init_logger();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/no-body"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"hello": "world"})))
        .mount(&server)
        .await;

    let mut path = PathBuf::new();
    path.push("tests/e2e_inputs/simple_markdown.md");
    let result = run(
        &path,
        &common::create_environment_variables(&server),
        &ExecuteOptions::new_update(
            true,
            UpdateMode::Overwrite,
            &[Detector::Timestamp, Detector::Timestamp],
        ),
    )
    .await
    .unwrap();

    assert_eq!(result, true);
}

#[tokio::test]
async fn markdown_with_multiple_sections() {
    common::init_logger();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/items/[0-9]+$"))
        .respond_with(|req: &Request| {
            let id = req.url.path().rsplit('/').next().unwrap_or("unknown");
            ResponseTemplate::new(200).set_body_json(json!({
                "id": id,
                "name": format!("Echo {}", id)
            }))
        })
        .mount(&server)
        .await;

    let mut path = PathBuf::new();
    path.push("tests/e2e_inputs/markdown_with_multiple_sections.md");
    let result = run(
        &path,
        &common::create_environment_variables(&server),
        &ExecuteOptions::new_update(
            true,
            UpdateMode::Overwrite,
            &[Detector::Timestamp, Detector::Timestamp],
        ),
    )
        .await
        .unwrap();

    assert_eq!(result, true);
}

#[tokio::test]
async fn status_pattern() {
    common::init_logger();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/good_status"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&server)
        .await;

    let mut path = PathBuf::new();
    path.push("tests/e2e_inputs/status_pattern.http");
    let result = run(
        &path,
        &common::create_environment_variables(&server),
        &ExecuteOptions::new_test(),
    )
        .await
        .unwrap();

    assert_eq!(result, true);
}
