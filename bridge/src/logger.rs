//! Provides logging utilities, used by application.

// use std::io;

use chrono::Local;
use slog::{o, Drain, FnValue, Level, Logger, PushFnValue, Record};
// use slog_async::Async;
// use slog_json::Json;

pub mod prelude {
    pub use slog::{slog_debug, slog_error, slog_info, slog_trace, slog_warn};
    pub use slog_scope::{debug, error, info, trace, warn};
}

// pub fn new<W1, W2>(w_out: W1, w_err: W2) -> Logger
// where
//     W1: io::Write + Send + 'static,
//     W2: io::Write + Send + 'static,
// {
//     let drain_out = Json::new(w_out).build();
//     let drain_err = Json::new(w_err).build();
//     let drain = Duplicate(
//         drain_out.filter(|r| !r.level().is_at_least(Level::Debug)),
//         drain_err.filter_level(Level::Debug),
//     )
//     .map(Fuse);
//
//     let drain = slog_envlogger::new(drain).fuse();
//     let drain = Async::new(drain).chan_size(2048).build().fuse();
//     let logger = Logger::root(drain, o!());
//
//     logger.new(o!(
//         "msg" => PushFnValue(move |record : &Record, ser| {
//             ser.emit(record.msg())
//         }),
//         "fqn" => PushFnValue(move |record : &Record, ser| {
//              ser.emit(format_args!("{}:{}", record.module(), record.line()))
//         }),
//         "time" => PushFnValue(move |_ : &Record, ser| {
//             ser.emit(Local::now().to_rfc3339())
//         }),
//         "lvl" => FnValue(move |rinfo : &Record| {
//             rinfo.level().as_str()
//         }),
//     ))
// }

pub fn term_new() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = drain.filter_level(Level::Info).fuse();

    let logger = slog::Logger::root(drain, o!());

    logger.new(o!(
        "msg" => PushFnValue(move |record : &Record, ser| {
            ser.emit(record.msg())
        }),
        "fqn" => PushFnValue(move |record : &Record, ser| {
             ser.emit(format_args!("{}:{}", record.module(), record.line()))
        }),
        "time" => PushFnValue(move |_ : &Record, ser| {
            ser.emit(Local::now().to_rfc3339())
        }),
        "lvl" => FnValue(move |rinfo : &Record| {
            rinfo.level().as_str()
        }),
    ))
}
