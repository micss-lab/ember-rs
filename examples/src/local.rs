extern crate std;

use std::borrow::Cow;

use env_logger::Env;

/// Initialize the logger with the given maximum log level.
pub fn init_logger(default_level: log::LevelFilter) {
    use std::io::Write;
    env_logger::Builder::from_env(
        Env::new().default_filter_or(minimize_depency_logs(level_to_str(default_level))),
    )
    .format(|buf, record| {
        const RESET: &str = "\u{001B}[0m";
        const RED: &str = "\u{001B}[31m";
        const GREEN: &str = "\u{001B}[32m";
        const YELLOW: &str = "\u{001B}[33m";
        const BLUE: &str = "\u{001B}[34m";
        const CYAN: &str = "\u{001B}[35m";

        let color = match record.level() {
            log::Level::Error => RED,
            log::Level::Warn => YELLOW,
            log::Level::Info => GREEN,
            log::Level::Debug => BLUE,
            log::Level::Trace => CYAN,
        };
        let reset = RESET;
        writeln!(
            buf,
            "{}{} - {}{}",
            color,
            record.level(),
            record.args(),
            reset
        )
    })
    .init()
}

fn level_to_str(level: log::LevelFilter) -> &'static str {
    use log::LevelFilter::*;

    match level {
        Error => "error",
        Warn => "warn",
        Info => "info",
        Debug => "debug",
        Trace => "trace",
        Off => "",
    }
}

fn minimize_depency_logs(current: &'static str) -> Cow<'static, str> {
    let mut result = Cow::from(current);
    let comma = !current.is_empty();

    *result.to_mut() += if comma { "," } else { "" };

    *result.to_mut() += "multipart=off,";
    *result.to_mut() += "ureq=off,";
    *result.to_mut() += "tiny_http=off,";

    result
}
