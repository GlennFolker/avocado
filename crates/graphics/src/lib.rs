use avocado_asset::prelude::*;
use avocado_core::prelude::*;

mod color;
mod img;

pub use color::*;
pub use img::*;

pub mod re_exports {
    pub use image;
}

pub mod prelude {
    pub use crate::{
        re_exports::*,
        GraphicsSubsystem,
        Color, Image,
    };

    pub use image::GenericImageView as _;
}

pub struct GraphicsSubsystem;
impl Subsystem for GraphicsSubsystem {
    fn init(app: &mut App) {
        app
            .asset::<Image>()
            .asset_loader::<Image>(ImageLoader);
    }
}
