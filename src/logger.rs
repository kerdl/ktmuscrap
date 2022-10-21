use colored::Colorize;
use log::{Record, Level, Metadata, Log, SetLoggerError, LevelFilter};


pub struct Logger;

impl Logger {
    pub fn init() -> Result<(), SetLoggerError>  {
        log::set_logger(&crate::LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Info))
    }
}
impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let date = chrono::Local::now();
            let fmt_date = date.format("%Y-%m-%d at %H:%M:%S")
                .to_string()
                .bright_cyan();

            println!("{} [{}] {}", 
                fmt_date, 
                record.level(), 
                record.args()
            );
        }
    }

    fn flush(&self) {}
}