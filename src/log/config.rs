use crate::incl::*;

pub struct LogConfig {
    pub formatter: Option<Box<dyn Fn(
        &mut env_logger::fmt::Formatter,
        &log::Record<'_>,
    ) -> io::Result<()> + Sync + Send>>,
    pub format_level: bool,
    pub format_module: bool,
    pub format_target: bool,
    pub format_indent: Option<usize>,
    pub format_time: Option<env_logger::fmt::TimestampPrecision>,
    pub format_suffix: Option<&'static str>,

    pub filter: log::LevelFilter,
    pub filter_modules: HashMap<&'static str, log::LevelFilter>,

    pub target: env_logger::fmt::Target,
    pub style: env_logger::fmt::WriteStyle,
}

impl LogConfig {
    fn level_filter() -> log::LevelFilter {
        if cfg!(debug_assertions) {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Warn
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
            format_time: Some(env_logger::fmt::TimestampPrecision::Seconds),
            format_suffix: None,

            filter: Self::level_filter(),
            filter_modules: {
                let mut map = HashMap::default();
                #[cfg(feature = "winit")]
                {
                    map.insert("wgpu", log::LevelFilter::Error);
                    map.insert("wgpu_core", log::LevelFilter::Error);
                    map.insert("wgpu_hal", log::LevelFilter::Error);
                    map.insert("naga", log::LevelFilter::Error);
                }

                map
            },

            target: env_logger::fmt::Target::Stdout,
            style: env_logger::fmt::WriteStyle::Auto,
        }
    }
}
