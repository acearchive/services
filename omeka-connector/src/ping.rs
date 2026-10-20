use std::time::Duration;

const PING_TIMEOUT: Duration = Duration::from_secs(10);

/// Ping a healthcheck endpoint, logging a warning on failure.
pub async fn send(url: &str) {
    let result = reqwest::Client::new()
        .get(url)
        .timeout(PING_TIMEOUT)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status);

    // The ping URL is a secret, so keep it out of the logs.
    if let Err(err) = result {
        log::warn!("Failed to ping healthcheck endpoint: {}", err.without_url());
    }
}
