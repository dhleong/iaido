use std::sync::Mutex;

use bounded_vec_deque::BoundedVecDeque;
use lazy_static::lazy_static;

use crate::editing::{text::TextLines, Buffer};

pub enum LogLevel {
    Info,
    Warn,
    Error,
}
struct LogEntry(LogLevel, String);

impl From<&LogEntry> for TextLines {
    fn from(entry: &LogEntry) -> Self {
        let LogEntry(level, ref text) = entry;
        match level {
            _ => TextLines::raw(text.clone()),
        }
    }
}

const MESSAGE_HISTORY_SIZE: usize = 200;

lazy_static! {
    static ref LOG_LINES: Mutex<BoundedVecDeque<LogEntry>> =
        Mutex::new(BoundedVecDeque::new(MESSAGE_HISTORY_SIZE));
}

pub fn push_log(level: LogLevel, text: String) {
    let mut lines = LOG_LINES.lock().unwrap();
    lines.push_back(LogEntry(level, text));
}

pub fn write_to_buffer(buffer: &mut Box<dyn Buffer>) {
    let lines = LOG_LINES.lock().unwrap();
    for line in lines.iter() {
        buffer.append(line.into());
    }
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {{
        crate::log::push_log($level, format!($($arg)*).to_string());
    }}
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        crate::log!(crate::log::LogLevel::Info, $($arg)*);
    }}
}
