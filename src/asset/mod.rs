use crate::incl::*;

mod asset;
mod event;
mod ext;
mod graph;
mod handle;
mod loader;
mod reader;
mod server;
mod sys;

pub use asset::*;
pub use event::*;
pub use ext::AppExt as _;
pub use graph::*;
pub use handle::*;
pub use loader::*;
pub use reader::*;
pub use server::*;
pub use sys::*;

pub struct AssetSubsystem;
impl Subsystem for AssetSubsystem {
    fn init(app: &mut App) {
        app
            .event::<AssetGraphDoneEvent>()
            .insert_res(AssetServer::new(Arc::new(DefaultAssetReader::default())))

            .stage_before(CoreStage::SysUpdate, AssetStage, SystemStage::parallel())

            .sys(CoreStage::SysPostUpdate, AssetServer::post_update_sys.at_end())
            .sys(CoreStage::SysPostUpdate, AssetGraph::update_sys.at_end());
    }
}
