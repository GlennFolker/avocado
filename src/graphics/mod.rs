use crate::core::prelude::*;
#[cfg(feature = "asset")]
use crate::asset::prelude::*;

mod bin_pack;
mod bin_page;
mod color;
mod img;

pub use bin_pack::*;
pub use bin_page::*;
pub use color::*;
pub use img::*;

pub mod re_exports {
    pub use ::image;
}

pub mod prelude {
    pub use crate::graphics::{
        re_exports::*,
        GraphicsSubsystem,
        Color, Image,
        BinPack, BinPage, BinRect,
    };

    pub use image::GenericImageView as _;
}

pub struct GraphicsSubsystem;
impl Subsystem for GraphicsSubsystem {
    fn init(app: &mut App) {
        #[cfg(feature = "asset")]
        app
            .asset::<Image>()
            .asset_loader::<Image>(ImageLoader);
    }
}
