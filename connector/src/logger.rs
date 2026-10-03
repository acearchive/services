use flexi_logger::{
    Age, Cleanup, Criterion, DeferredNow, FileSpec, FlexiLoggerError, Logger, LoggerHandle, Naming,
    WriteMode,
};
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
        .log_to_file(FileSpec::default().directory("logs"))
        .format(format)
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(14),
        )
        .write_mode(WriteMode::Async)
        .start()
}
