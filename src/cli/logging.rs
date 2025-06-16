use log::LevelFilter;
use simple_logger::SimpleLogger;

/// Set the log level for the application
pub fn set_log_level(level: LevelFilter) {
    // Reset the logger with the new level
    let _ = SimpleLogger::new()
        .with_level(level)
        .init();
}

/// Initialize logging with the specified level
pub fn init_logging(debug: bool) -> LevelFilter {
    let log_level = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    
    SimpleLogger::new()
        .with_level(log_level)
        .init()
        .unwrap();
        
    log_level
}

/// Configure backtrace if trace is enabled
pub fn configure_backtrace(trace: bool) {
    if trace {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
} 
