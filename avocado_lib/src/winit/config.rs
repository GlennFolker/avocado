use crate::incl::*;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ClearColor(pub Color);

pub type WindowCreationResult = Result<winit::Window, WindowCreationError>;

pub enum WindowPosition {
    Auto,
    Center,
    Pos(winit::Position)
}

#[derive(Resource)]
/// Configuration to create a desktop/browser window. Most fields are unused for mobile surfaces.
pub struct WindowConfig {
    pub size: Option<winit::Size>,
    pub min_size: Option<winit::Size>,
    pub max_size: Option<winit::Size>,
    pub pos: WindowPosition,

    pub title: String,
    pub resizable: bool,
    /// `None` to disable fullscreen, `false` to enable exclusive fullscreen, `true` to enable borderless fullscreen.
    pub fullscreen: Option<bool>,
    pub maximized: bool,

    pub visible: bool,
    pub transparent: bool,
    pub decorations: bool,
    pub always_on_top: bool,
    pub vsync: bool,
}

impl WindowConfig {
    pub fn visible_sys(window: NonSend<WinitWindow>, config: Res<WindowConfig>) {
        window.set_visible(config.visible);
    }

    pub fn create<T>(&self, event_loop: &winit::EventLoopWindowTarget<T>) -> WindowCreationResult {
        let monitor = event_loop.available_monitors().find(|_| true).ok_or(WindowCreationError::NoMonitor)?;
        let mode = monitor.video_modes().find(|_| true);

        let mut builder = winit::WindowBuilder::new();
        builder = if let Some(size) = self.size { builder.with_inner_size(size) } else { builder };
        builder = if let Some(min_size) = self.min_size { builder.with_min_inner_size(min_size) } else { builder };
        builder = if let Some(max_size) = self.max_size { builder.with_max_inner_size(max_size) } else { builder };

        builder = if let Some(borderless) = self.fullscreen {
            if borderless || mode.is_none() {
                if !borderless {
                    log::warn!("No available video mode found; using borderless fullscreen instead.");
                }

                builder.with_fullscreen(Some(winit::Fullscreen::Borderless(Some(monitor.clone()))))
            } else {
                builder.with_fullscreen(Some(winit::Fullscreen::Exclusive(mode.unwrap())))
            }
        } else {
            builder
        };

        match builder
            .with_title(self.title.clone())
            .with_resizable(self.resizable)
            .with_maximized(self.maximized)
            .with_visible(false)
            .with_transparent(self.transparent)
            .with_decorations(self.decorations)
            .with_always_on_top(self.always_on_top)

            .build(&event_loop)
        {
            Ok(window) => {
                if self.fullscreen.is_none() {
                    match self.pos {
                        WindowPosition::Auto => window.set_outer_position(monitor.position()),
                        WindowPosition::Center => {
                            let pos = monitor.position().cast::<f64>();
                            let monitor_size = monitor.size().cast::<f64>();
                            let size = window.outer_size().cast::<f64>();
                            window.set_outer_position(winit::PhysicalPosition {
                                x: (pos.x + monitor_size.width / 2.) - (size.width / 2.),
                                y: (pos.y + monitor_size.height / 2.) - (size.height / 2.)
                            });
                        },
                        WindowPosition::Pos(rel) => {
                            let monitor_pos = monitor.position().cast::<f64>();
                            let pos = rel.to_physical::<f64>(window.scale_factor());
                            window.set_outer_position(winit::PhysicalPosition {
                                x: monitor_pos.x + pos.x,
                                y: monitor_pos.y + pos.y
                            });
                        }
                    }
                }

                Ok(window)
            },

            Err(os_error) => Err(WindowCreationError::Os(os_error)),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            size: Some(winit::Size::Logical(winit::LogicalSize { width: 800., height: 600. })),
            min_size: None,
            max_size: None,
            pos: WindowPosition::Center,

            title: "AVocado Application".to_string(),
            resizable: true,
            fullscreen: None,
            maximized: false,

            visible: true,
            transparent: false,
            decorations: true,
            always_on_top: false,
            vsync: true,
        }
    }
}

#[derive(Error, Debug)]
pub enum WindowCreationError {
    #[error("No available monitor found")]
    NoMonitor,
    #[error("OS error")]
    Os(#[from] winit::OsError),
}
