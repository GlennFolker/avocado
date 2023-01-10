use crate::core::prelude::*;
use env_logger::Builder;

mod config;

pub use config::*;

pub mod re_exports {
    pub use ::env_logger;
}

pub mod prelude {
    pub use crate::log::{
        LogSubsystem,
        LogConfig,
    };
}

pub struct LogSubsystem;
impl Subsystem for LogSubsystem {
    fn init(app: &mut App) {
        let config = match app.remove_res_ns::<LogConfig>() {
            Some(config) => config,
            None => LogConfig::default(),
        };

        let mut builder = Builder::new();
        builder
            .format_level(config.format_level)
            .format_module_path(config.format_module)
            .format_target(config.format_target)
            .format_indent(config.format_indent)
            .format_timestamp(config.format_time)

            .filter_level(config.filter)

            .target(config.target)
            .write_style(config.style);

        if let Some(formatter) = config.formatter {
            builder.format(formatter);
        }

        if let Some(suffix) = config.format_suffix {
            builder.format_suffix(suffix);
        }

        for (module, level) in config.filter_modules {
            builder.filter_module(module, level);
        }

        match builder.parse_default_env().try_init() {
            Ok(_) => log::info!("Successfully initialized logger"),
            Err(err) => log::error!("Couldn't initialize logger: {}", err),
        }
    }
}
