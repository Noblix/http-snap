use std::collections::HashMap;
use std::sync::Once;
use log::LevelFilter;
use http_snap::types::Value;

static INIT: Once = Once::new();

// Initializes the logger only once.
pub fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(LevelFilter::Error)
            .init();
    });
}

pub fn create_environment_variables(server: &mockito::Server) -> HashMap<String, Value> {
    return HashMap::from([(
        "test_host".to_string(),
        Value::from(server.host_with_port()),
    )]);
}