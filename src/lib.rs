use avocado_core::{
    App, Subsystem,
};

#[cfg(feature = "asset")]
pub mod asset {
    pub use avocado_asset::prelude::*;
}
#[cfg(feature = "core")]
pub mod core {
    pub use avocado_core::prelude::*;
}
#[cfg(feature = "g2d")]
pub mod g2d {
    pub use avocado_g2d::prelude::*;
}
#[cfg(feature = "graphics")]
pub mod graphics {
    pub use avocado_graphics::prelude::*;
}
#[cfg(feature = "input")]
pub mod input {
    pub use avocado_input::prelude::*;
}
#[cfg(feature = "log")]
pub mod log {
    pub use avocado_log::prelude::*;
}
#[cfg(feature = "utils")]
pub mod utils {
    pub use avocado_utils::prelude::*;
}
#[cfg(feature = "winit")]
pub mod winit {
    pub use avocado_winit::prelude::*;
}

pub mod prelude {
    pub use crate::AVocado;

    #[cfg(feature = "asset")]
    pub use crate::asset::*;
    #[cfg(feature = "core")]
    pub use crate::core::*;
    #[cfg(feature = "g2d")]
    pub use crate::g2d::*;
    #[cfg(feature = "graphics")]
    pub use crate::graphics::*;
    #[cfg(feature = "input")]
    pub use crate::input::*;
    #[cfg(feature = "log")]
    pub use crate::log::*;
    #[cfg(feature = "utils")]
    pub use crate::utils::*;
    #[cfg(feature = "winit")]
    pub use crate::winit::*;
}

pub struct AVocado;

#[cfg(feature = "core")]
impl Subsystem for AVocado {
    fn init(app: &mut App) {
        #[cfg(feature = "log")]
        app.init::<log::LogSubsystem>();

        app.init::<core::CoreSubsystem>();

        #[cfg(feature = "asset")]
        app.init::<asset::AssetSubsystem>();
        #[cfg(feature = "graphics")]
        app.init::<graphics::GraphicsSubsystem>();
        #[cfg(feature = "winit")]
        app.init::<winit::WinitSubsystem>();
        #[cfg(feature = "g2d")]
        app.init::<g2d::G2dSubsystem>();
    }
}
