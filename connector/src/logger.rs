use flexi_logger::{DeferredNow, FlexiLoggerError, Logger, LoggerHandle, WriteMode};
use log::Record;
use std::io::Write;

fn format(w: &mut dyn Write, now: &mut DeferredNow, record: &Record) -> std::io::Result<()> {
    write!(
        w,
        "{} [{}] {}",
        now.format_rfc3339(),
        record.level(),
        record.args()
    )
}

pub fn init_logger() -> Result<LoggerHandle, FlexiLoggerError> {
    Logger::try_with_str("info")?
        .log_to_stderr()
        .format(format)
        .write_mode(WriteMode::Async)
        .start()
}
