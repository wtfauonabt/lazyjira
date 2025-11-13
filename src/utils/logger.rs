use log::LevelFilter;

/// Initialize the logger with default settings
pub fn init_logger(level: LevelFilter) {
    env_logger::Builder::from_default_env()
        .filter_level(level)
        .init();
}
