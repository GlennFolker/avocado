use crate::incl::*;

#[derive(Resource, Default)]
pub struct AssetServer {
    ref_channels: Arc<RwLock<HashMap<Uuid, Box<dyn RefPipe>>>>,
    asset_channels: Arc<RwLock<HashMap<Uuid, Box<dyn AssetPipe>>>>,
}

impl AssetServer {
    pub fn update_sys<T: Asset>(
        server: ResMut<AssetServer>,
        mut events: EventWriter<AssetEvent<T>>, mut assets: ResMut<Assets<T>>
    ) {
        server.update(&mut events, &mut assets);
    }

    pub fn register<T: Asset>(&mut self) -> Assets<T> {
        let ref_channel = RefChannel::<T>::default();
        let ref_change = ref_channel.sender.clone();

        if self.ref_channels.write().insert(T::TYPE_UUID, Box::new(ref_channel)).is_some() {
            panic!("Asset {} is already registered", type_name::<T>());
        }

        if self.asset_channels.write().insert(T::TYPE_UUID, Box::new(AssetChannel::<T>::default())).is_some() {
            panic!("Asset {} is already registered", type_name::<T>());
        }

        Assets::<T>::new(ref_change)
    }

    pub fn load<T: Asset>(&mut self, handle_path: impl Into<Cow<'static, Path>>) -> Handle<T> {
        let ref_channels = self.ref_channels.read();
        let Some(refc) = ref_channels.get(&T::TYPE_UUID) else {
            panic!("Asset {} is not registered", type_name::<T>());
        };
        let refc = refc.downcast_ref::<RefChannel<T>>().unwrap();

        Handle::strong(handle_path.into(), refc.sender.clone())
    }

    pub fn update<T: Asset>(&self, events: &mut EventWriter<AssetEvent<T>>, assets: &mut Assets<T>) {
        let ref_channels = self.ref_channels.read();
        let asset_channels = self.asset_channels.read();

        let Some(refc) = ref_channels.get(&T::TYPE_UUID) else {
            panic!("Asset {} is not registered", type_name::<T>());
        };
        let refc = refc.downcast_ref::<RefChannel<T>>().unwrap();

        let Some(pipe) = asset_channels.get(&T::TYPE_UUID) else {
            panic!("Asset {} is not registered", type_name::<T>());
        };
        let pipe = pipe.downcast_ref::<AssetChannel<T>>().unwrap();

        loop {
            match refc.receiver.try_recv() {
                Ok(change) => match change {
                    RefChange::Incr(handle) => assets.incr_count(handle.handle_path.clone(), 1),
                    RefChange::Decr(handle) => {
                        assets.incr_count(handle.handle_path.clone(), -1);
                        if assets.count(&handle.handle_path) <= 0 {
                            if let Err(msg) = pipe.sender.send(AssetLife::Removed(handle.handle_path.clone())) {
                                log::warn!("Couldn't send asset removal signal for {:?}: {}", &handle.handle_path, msg);
                            }
                        }
                    },
                },
                Err(err) => match err {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => panic!("Ref channel {} disconnected", type_name::<T>()),
                },
            }
        }

        loop {
            match pipe.receiver.try_recv() {
                Ok(life) => match life {
                    AssetLife::Created(path, asset) => if assets.count(&path) > 0 {
                        assets.add_direct(events, path, asset);
                    },
                    AssetLife::Removed(path) => assets.remove_direct(events, path),
                },
                Err(err) => match err {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => panic!("Asset channel {} disconnected", type_name::<T>()),
                },
            }
        }
    }
}
