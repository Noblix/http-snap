use http_snap::{run};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Arguments should come from CLI
    return run(
        //"http-examples/todo-app/can_manage_todo_items_with_api_wip.http"
        //"http-examples/todo-app/cannot_create_todo_item_with_no_text.http"
        "http-examples/todo-app/cannot_complete_todo_item_multiple_times.http"
        //"http-examples/todo-app/cannot_complete_deleted_todo_item.http"
        //"http-examples/todo-app/cannot_delete_todo_item_multiple_times.http"
        //"http-examples/todo-app/sign_in.http"
    ,
    false)
    .await;
}
