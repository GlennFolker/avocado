use crate::{
    core::prelude::*,
    asset::{
        Asset, AssetLoader, AssetServer,
    },
};

pub trait AppExt {
    fn asset<T: Asset>(&mut self) -> &mut Self;
    fn asset_loader<T: Asset>(&mut self, loader: impl AssetLoader) -> &mut Self;
}

impl AppExt for App {
    fn asset<T: Asset>(&mut self) -> &mut Self {
        let assets = self.res_mut::<AssetServer>().unwrap().register::<T>();
        self
            .insert_res(assets)
            .sys(CoreStage::SysUpdate, AssetServer::update_sys::<T>)
    }

    fn asset_loader<T: Asset>(&mut self, loader: impl AssetLoader) -> &mut Self {
        self.res_mut::<AssetServer>().unwrap().set_loader::<T>(loader);
        self
    }
}
