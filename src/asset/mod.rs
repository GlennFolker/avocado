use crate::core::prelude::*;
use std::sync::Arc;

mod asset;
mod event;
mod ext;
mod graph;
mod handle;
mod loader;
mod reader;
mod server;

pub use asset::*;
pub use event::*;
pub use ext::*;
pub use graph::*;
pub use handle::*;
pub use loader::*;
pub use reader::*;
pub use server::*;

pub mod prelude {
    pub use crate::asset::{
        AssetSubsystem, AppExt as _,

        Asset, AssetDyn, Assets, AssetServer, AssetLoader, AssetReader,
        Handle, HandleDyn, AssetState,
        AssetData, NoAssetData, AssetLoadSync,
        AssetGraph, AssetGraphIn, AssetGraphOut, AssetGraphResult, AssetGraphBuilder, AssetLoaderError,
        AssetGraphDoneEvent,
    };
}

pub struct AssetSubsystem;
impl Subsystem for AssetSubsystem {
    fn init(app: &mut App) {
        app
            .event::<AssetGraphDoneEvent>()
            .insert_res(AssetServer::new(Arc::new(DefaultAssetReader::default())))

            .sys(CoreStage::SysPostUpdate, AssetServer::post_update_sys.at_end())
            .sys(CoreStage::SysPostUpdate, AssetGraph::update_sys.at_end());
    }
}
