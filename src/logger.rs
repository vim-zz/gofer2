use log::LevelFilter;
use oslog::OsLogger;

/// Initializes the logger for the application.
/// Call this early in main.
pub fn init_logger() {
    OsLogger::new("com.example.basicmenubarapp")
        .level_filter(LevelFilter::Debug)
        .init()
        .unwrap();
}
