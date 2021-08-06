use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter::Info;
use std::io;

pub fn setup_logging() -> Result<(), fern::InitError> {
    let config = fern::Dispatch::new();
    let colors_config = ColoredLevelConfig::new()
        .info(Color::BrightCyan)
        .debug(Color::White)
        .warn(Color::BrightYellow)
        .error(Color::Red);

    config
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                colors_config.color(record.level()),
                message
            ))
        })
        // setting default log level to "INFO"
        .level(Info)
        // TODO: add a logger for "TRACE" - with lesser noise preferrably
        .level_for("graphql-engine-rs", Info)
        .chain(io::stdout())
        .apply()?;

    Ok(())
}
