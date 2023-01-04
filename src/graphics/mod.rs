use crate::incl::*;

mod color;
mod img;

pub use color::*;
pub use img::*;

pub struct GraphicsSubsystem;
impl Subsystem for GraphicsSubsystem {
    fn init(app: &mut App) {
        #[cfg(feature = "asset")]
        app
            .asset::<Image>()
            .asset_loader::<Image>(ImageLoader);
    }
}
