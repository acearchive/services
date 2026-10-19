#![forbid(unsafe_code)]

use acearchive_data_connector::{config, init_config, init_logger, router, run_export_scheduler};

#[tokio::main]
async fn main() {
    init_config().expect("Failed to load config.");
    let _logger = init_logger().expect("Failed to start logger.");

    let port = config::port();
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    match config::data_export() {
        Some(export_config) => {
            log::info!("Data exports enabled.");
            tokio::spawn(run_export_scheduler(export_config.to_owned()));
        }
        None => log::info!("Data exports disabled."),
    }

    log::info!("Server starting.");

    axum::serve(listener, router())
        .await
        .expect("Failed to start server.");
}
