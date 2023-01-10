use bevy_utils::HashMap;
use env_logger::{
    fmt::{
        Formatter,
        Target, TimestampPrecision, WriteStyle,
    },
};
use log::{
    LevelFilter,
    Record,
};
use std::io;

pub struct LogConfig {
    pub formatter: Option<Box<dyn Fn(
        &mut Formatter,
        &Record<'_>,
    ) -> io::Result<()> + Sync + Send>>,
    pub format_level: bool,
    pub format_module: bool,
    pub format_target: bool,
    pub format_indent: Option<usize>,
    pub format_time: Option<TimestampPrecision>,
    pub format_suffix: Option<&'static str>,

    pub filter: LevelFilter,
    pub filter_modules: HashMap<&'static str, LevelFilter>,

    pub target: Target,
    pub style: WriteStyle,
}

impl LogConfig {
    fn level_filter() -> LevelFilter {
        if cfg!(debug_assertions) {
            LevelFilter::Trace
        } else {
            LevelFilter::Warn
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            formatter: None,
            format_level: true,
            format_module: true,
            format_target: false,
            format_indent: Some(4),
            format_time: Some(TimestampPrecision::Seconds),
            format_suffix: None,

            filter: Self::level_filter(),
            filter_modules: {
                #[allow(unused_mut)]
                let mut map = HashMap::default();

                // Winit/WGPU log filters.
                map.insert("wgpu", LevelFilter::Error);
                map.insert("wgpu_core", LevelFilter::Error);
                map.insert("wgpu_hal", LevelFilter::Error);
                map.insert("naga", LevelFilter::Error);

                map
            },

            target: Target::Stdout,
            style: WriteStyle::Auto,
        }
    }
}
