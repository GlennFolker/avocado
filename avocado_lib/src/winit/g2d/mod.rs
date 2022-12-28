use crate::incl::*;

mod atlas;

pub use atlas::*;

pub struct Winit2dSubsystem;
impl Subsystem for Winit2dSubsystem {
    fn init(app: &mut App) {
        app
            .asset::<TextureAtlas>()
            .asset_loader::<TextureAtlas>(TextureAtlasLoader);
    }
}
