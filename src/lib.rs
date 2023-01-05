#[cfg(feature = "asset")]
use avocado_asset::prelude::*;
#[cfg(feature = "core")]
use avocado_core::prelude::*;
#[cfg(feature = "g2d")]
use avocado_g2d::prelude::*;
#[cfg(feature = "graphics")]
use avocado_graphics::prelude::*;
#[cfg(feature = "input")]
use avocado_input::prelude::*;
#[cfg(feature = "log")]
use avocado_log::prelude::*;
#[cfg(feature = "winit")]
use avocado_winit::prelude::*;

pub mod prelude {
    pub use crate::AVocado;

    #[cfg(feature = "asset")]
    pub use avocado_asset::prelude::*;
    #[cfg(feature = "core")]
    pub use avocado_core::prelude::*;
    #[cfg(feature = "g2d")]
    pub use avocado_g2d::prelude::*;
    #[cfg(feature = "graphics")]
    pub use avocado_graphics::prelude::*;
    #[cfg(feature = "input")]
    pub use avocado_input::prelude::*;
    #[cfg(feature = "log")]
    pub use avocado_log::prelude::*;
    #[cfg(feature = "utils")]
    pub use avocado_utils::prelude::*;
    #[cfg(feature = "winit")]
    pub use avocado_winit::prelude::*;
}

pub struct AVocado;

#[cfg(feature = "core")]
impl Subsystem for AVocado {
    fn init(app: &mut App) {
        #[cfg(feature = "log")]
        app.init::<LogSubsystem>();

        app.init::<CoreSubsystem>();

        #[cfg(feature = "asset")]
        app.init::<AssetSubsystem>();
        #[cfg(feature = "graphics")]
        app.init::<GraphicsSubsystem>();
        #[cfg(feature = "winit")]
        app.init::<WinitSubsystem>();
        #[cfg(feature = "g2d")]
        app.init::<G2dSubsystem>();
    }
}
