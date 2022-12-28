use crate::incl::*;

mod asset;
mod ext;
mod handle;
mod loader;
mod reader;
mod server;
mod sys;

pub use asset::*;
pub use ext::AppExt as _;
pub use handle::*;
pub use loader::*;
pub use reader::*;
pub use server::*;
pub use sys::*;

pub struct AssetSubsystem;
impl Subsystem for AssetSubsystem {
    fn init(app: &mut App) {
        app
            .stage_before(CoreStage::SysUpdate, AssetStage, SystemStage::parallel())
            .insert_res(AssetServer::new(Arc::new(DefaultAssetReader::default())))

            .sys(CoreStage::SysPostUpdate, AssetServer::post_update_sys.at_end());
    }
}
