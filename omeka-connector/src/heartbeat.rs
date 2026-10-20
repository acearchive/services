use chrono::Utc;

use crate::{config::HeartbeatConfig, ping};

/// Ping the heartbeat endpoint on the configured schedule, forever.
pub async fn run_scheduler(heartbeat_config: HeartbeatConfig) {
    loop {
        let now = Utc::now();

        let next = match heartbeat_config.cron_spec.find_next_occurrence(&now, false) {
            Ok(next) => next,
            Err(err) => {
                log::error!("No upcoming heartbeat matches the cron spec: {}", err);
                return;
            }
        };

        log::debug!("Next heartbeat scheduled for {}.", next.to_rfc3339());

        tokio::time::sleep((next - now).to_std().unwrap_or_default()).await;

        ping::send(heartbeat_config.ping_url.as_str()).await;
    }
}
