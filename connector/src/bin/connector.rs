#![forbid(unsafe_code)]

use acearchive_data_connector::{init_config, init_logger};

fn main() {
    init_config().expect("Failed to load config.");
    let _logger = init_logger().expect("Failed to start logger.");

    log::info!("Server starting.");
}
