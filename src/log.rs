use serde::{Deserialize, Serialize};

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    Trace = 0,
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl From<LogLevel> for proxy_wasm::types::LogLevel {
    fn from(level: LogLevel) -> Self {
        use LogLevel::*;

        match level {
            Critical => Self::Critical,
            Warn => Self::Warn,
            Info => Self::Info,
            Error => Self::Error,
            Debug => Self::Debug,
            Trace => Self::Trace,
        }
    }
}

pub trait IdentLogger {
    fn ident(&self) -> &str;
}

impl<'a> IdentLogger for &'a str {
    fn ident(&self) -> &str {
        self
    }
}

#[macro_export(local_inner_macros)]
macro_rules! log {
    ($ident:expr, $lvl:expr, $($arg:tt)+) => ({
        $crate::log::with_ident($ident, core::format_args!($($arg)+), $lvl)
    })
}

#[macro_export(local_inner_macros)]
macro_rules! trace {
    ($ident:expr, $($arg:tt)+) => (
        log!($ident, crate::log::LogLevel::Trace, $($arg)+)
    )
}

#[macro_export(local_inner_macros)]
macro_rules! debug {
    ($ident:expr, $($arg:tt)+) => (
        log!($ident, crate::log::LogLevel::Debug, $($arg)+)
    )
}

#[macro_export(local_inner_macros)]
macro_rules! info {
    ($ident:expr, $($arg:tt)+) => (
        log!($ident, crate::log::LogLevel::Info, $($arg)+)
    )
}

#[macro_export(local_inner_macros)]
macro_rules! warn {
    ($ident:expr, $($arg:tt)+) => (
        log!($ident, crate::log::LogLevel::Warn, $($arg)+)
    )
}

#[macro_export(local_inner_macros)]
macro_rules! error {
    ($ident:expr, $($arg:tt)+) => (
        log!($ident, crate::log::LogLevel::Error, $($arg)+)
    )
}

#[macro_export(local_inner_macros)]
macro_rules! critical {
    ($ident:expr, $($arg:tt)+) => (
        log!($ident, crate::log::LogLevel::Critical, $($arg)+)
    )
}

pub fn with_ident(ctx: &dyn IdentLogger, args: core::fmt::Arguments, level: LogLevel) {
    let ident = ctx.ident();

    proxy_wasm::hostcalls::log(level.into(), &format!("{}: {}", ident, args)).unwrap();
}
