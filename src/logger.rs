use log::LevelFilter;
use oslog::OsLogger;

/// Initializes the logger for the application.
pub fn init_logger() {
    OsLogger::new("com.1000ants.gofer2")
        .level_filter(LevelFilter::Debug)
        .init()
        .unwrap();
}
