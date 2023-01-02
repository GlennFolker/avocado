use crate::incl::*;

mod atlas;
mod batch;

pub use atlas::*;
pub use batch::*;

pub struct Winit2dSubsystem;
impl Subsystem for Winit2dSubsystem {
    fn init(app: &mut App) {
        app
            .asset::<TextureAtlas>()
            .asset_loader::<TextureAtlas>(TextureAtlasLoader);
    }
}
