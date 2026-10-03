#![forbid(unsafe_code)]

fn main() {
    let _logger = init_logging().expect("failed to start logger");
    log::info!("Server starting.");
}
