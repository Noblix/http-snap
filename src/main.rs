use http_snap::{run};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Arguments should come from CLI
    return run(
        "http-examples/todo-app/cannot_create_todo_item_with_no_text.http"
    )
    .await;
}
