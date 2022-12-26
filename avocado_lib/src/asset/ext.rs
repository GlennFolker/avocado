use crate::incl::*;

pub trait AppExt {
    fn asset<T: Asset>(&mut self) -> &mut Self;
}

impl AppExt for App {
    fn asset<T: Asset>(&mut self) -> &mut Self {
        let assets = self.res_mut::<AssetServer>().unwrap().register::<T>();
        self
            .insert_res(assets)
            .event::<AssetEvent<T>>()
            .sys(CoreStage::PreUpdate, AssetServer::update_sys::<T>)
    }
}
