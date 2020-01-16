include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use log::{self, Level, LevelFilter, Log, Metadata, Record};

use lazy_static::lazy_static;
use std::ffi::CString;

lazy_static! {
    static ref LOGGER: GGLogger = GGLogger;
}

#[derive(Default)]
pub struct GGLogger;

impl Log for GGLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            to_gg_log(record)
        }
    }

    fn flush(&self) {}
}

pub fn init_log(max_level: LevelFilter) {
    log::set_max_level(max_level);
    log::set_logger(&*LOGGER).expect("GGLogger implementation could not be set as logger");
}

fn to_gg_log(record: &Record) {
    let formatted = format!("{} -- {}", record.target(), record.args());
    let bytes = formatted.into_bytes();

    let c_to_print = CString::new(bytes.as_slice()).expect("CString: new failed");
    let level = to_gg_log_level(&record.level());
    unsafe {
        gg_log(level, c_to_print.as_ptr());
    }
}

fn to_gg_log_level(l: &Level) -> gg_log_level {
    match l {
        Level::Info => gg_log_level_GG_LOG_INFO,
        Level::Warn => gg_log_level_GG_LOG_WARN,
        Level::Error => gg_log_level_GG_LOG_ERROR,
        _ => gg_log_level_GG_LOG_DEBUG,
    }
}
