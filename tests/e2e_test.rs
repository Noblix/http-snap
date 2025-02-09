use reqwest::Client;

#[tokio::test]
async fn mock_simple_get() {
    let mut server = mockito::Server::new_async().await;

    // Use one of these addresses to configure your client
    let url = server.url();

    // Create a mock
    server
        .mock("GET", "/hello")
        .with_status(201)
        .with_header("content-type", "text/plain")
        .with_header("x-api-key", "1234")
        .with_body("world")
        .create();

    let client = Client::new();
    let response = client.get(format!("{url}/hello")).send().await.unwrap();

    assert_eq!(response.text().await.unwrap(), "world");
}
