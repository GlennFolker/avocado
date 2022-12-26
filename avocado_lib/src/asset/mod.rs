use crate::incl::*;

mod asset;
mod ext;
mod handle;
mod server;

pub use asset::*;
pub use ext::AppExt as _;
pub use handle::*;
pub use server::*;

pub struct AssetSubsystem;
impl Subsystem for AssetSubsystem {
    fn init(app: &mut App) {
        app.init_res::<AssetServer>();
    }
}
