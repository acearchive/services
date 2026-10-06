#![forbid(unsafe_code)]

use acearchive_data_connector::{config, init_config, init_logger, router};

#[tokio::main]
async fn main() {
    init_config().expect("Failed to load config.");
    let _logger = init_logger().expect("Failed to start logger.");

    let port = config::port().expect("Failed to get port number from config.");
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    log::info!("Server starting.");

    axum::serve(listener, router())
        .await
        .expect("Failed to start server.");
}
