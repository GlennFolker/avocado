use crate::incl::*;

#[derive(Resource)]
pub struct AssetServer {
    reader: Arc<dyn AssetReader>,

    ref_channels: HashMap<Uuid, Box<dyn RefPipe>>,
    asset_channels: HashMap<Uuid, Arc<dyn AssetPipe>>,

    states: HashMap<Uuid, Arc<RwLock<HashMap<Cow<'static, Path>, AssetState>>>>,
    loaders: HashMap<Uuid, Arc<dyn AssetLoader>>,
    changed: HashSet<Cow<'static, Path>>,
}

impl AssetServer {
    pub fn update_sys<T: Asset>(
        mut server: ResMut<AssetServer>,
        mut events: EventWriter<AssetEvent<T>>, mut assets: ResMut<Assets<T>>
    ) {
        server.update(&mut events, &mut assets);
    }

    pub fn new(reader: Arc<dyn AssetReader>) -> Self {
        Self {
            reader,

            ref_channels: HashMap::default(),
            asset_channels: HashMap::default(),

            states: HashMap::default(),
            loaders: HashMap::default(),
            changed: HashSet::default(),
        }
    }

    pub fn register<T: Asset>(&mut self) -> Assets<T> {
        let ref_channel = RefChannel::<T>::default();
        let ref_change = ref_channel.sender.clone();

        if self.ref_channels.insert(T::TYPE_UUID, Box::new(ref_channel)).is_some() {
            panic!("Asset {} is already registered", type_name::<T>());
        }

        if self.asset_channels.insert(T::TYPE_UUID, Arc::new(AssetChannel::<T>::default())).is_some() {
            panic!("Asset {} is already registered", type_name::<T>());
        }

        if self.states.insert(T::TYPE_UUID, Arc::default()).is_some() {
            panic!("Asset {} is already registered", type_name::<T>());
        }

        Assets::<T>::new(ref_change)
    }

    pub fn set_loader<T: Asset>(&mut self, loader: impl AssetLoader) -> Option<Arc<dyn AssetLoader>> {
        self.loaders.insert(T::TYPE_UUID, Arc::new(loader))
    }

    pub fn get_loader<T: Asset>(&self) -> Arc<dyn AssetLoader> {
        Arc::clone(self.loaders.get(&T::TYPE_UUID).expect(&format!("No asset loader set up for asset {}", type_name::<T>())))
    }

    pub fn load<T: Asset>(&mut self, handle_path: impl Into<Cow<'static, Path>>) -> Handle<T> {
        self.load_with(handle_path, None::<NoAssetData>)
    }

    pub fn load_with<T: Asset>(&mut self, handle_path: impl Into<Cow<'static, Path>>, data: Option<impl AssetData>) -> Handle<T> {
        let sender = &Self::get::<_, T>(&self.ref_channels).downcast_ref::<RefChannel<T>>().unwrap().sender;
        let state = Self::get::<_, T>(&self.states);
        let handle_path = handle_path.into();

        if {
            let mut should_load = false;

            let mut state = state.write();
            if !state.contains_key(&handle_path) {
                state.insert(handle_path.clone(), AssetState::Loading);
                should_load = true;
            }

            should_load
        } {
            let async_path = handle_path.clone();
            let state = Arc::clone(&state);
            let reader = Arc::clone(&self.reader);
            let pipe = Arc::clone(Self::get::<_, T>(&self.asset_channels));

            let loader = self.get_loader::<T>();
            let data: Option<Box<dyn AssetData>> = match data {
                Some(data) => Some(Box::new(data)),
                None => None,
            };

            IoTaskPool::get()
                .spawn(async move {
                    match loader.load(
                        reader, async_path.clone(),
                        data,
                    ) {
                        Ok(asset) => {
                            let sender = &pipe.downcast_ref::<AssetChannel<T>>().unwrap().sender;
                            if let Err(msg) = sender.send(AssetLife::Created(async_path.clone(), *asset.downcast::<T>().unwrap())) {
                                log::warn!("Couldn't send asset creation signal for {:?}: {}", &async_path, msg);
                            }
                        },
                        Err(err) => {
                            let msg = format!("{:?}", err);
                            log::error!("Couldn't load asset {:?}: {:?}", &async_path, &msg);

                            let mut state = state.write();
                            state.insert(async_path, AssetState::Errored(msg));
                        }
                    }
                })
                .detach();
        }

        Handle::strong(handle_path, sender.clone())
    }

    pub fn update<T: Asset>(&mut self, events: &mut EventWriter<AssetEvent<T>>, assets: &mut Assets<T>) {
        let refc = &Self::get::<_, T>(&self.ref_channels).downcast_ref::<RefChannel<T>>().unwrap().receiver;
        let pipe = Self::get::<_, T>(&self.asset_channels).downcast_ref::<AssetChannel<T>>().unwrap();

        self.changed.clear();
        loop {
            match refc.try_recv() {
                Ok(change) => match change {
                    RefChange::Incr(handle) => {
                        assets.incr_count(handle.handle_path.clone(), 1);
                        self.changed.insert(handle.handle_path.clone());
                    },
                    RefChange::Decr(handle) => {
                        assets.incr_count(handle.handle_path.clone(), -1);
                        self.changed.insert(handle.handle_path.clone());
                    },
                },
                Err(err) => match err {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => panic!("Ref channel {} disconnected", type_name::<T>()),
                },
            }
        }

        let mut state = Self::get::<_, T>(&self.states).write();
        for path in self.changed.iter() {
            if assets.count(&path) <= 0 {
                state.remove(path);
                if let Err(msg) = pipe.sender.send(AssetLife::Removed(path.clone())) {
                    log::warn!("Couldn't send asset removal signal for {:?}: {}", &path, msg);
                }
            }
        }

        loop {
            match pipe.receiver.try_recv() {
                Ok(life) => match life {
                    AssetLife::Created(path, asset) => if assets.count(&path) > 0 {
                        assets.add_direct(events, path, asset);
                    },
                    AssetLife::Removed(path) => {
                        state.remove(&path);
                        assets.remove_direct(events, path);
                    },
                },
                Err(err) => match err {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => panic!("Asset channel {} disconnected", type_name::<T>()),
                },
            }
        }
    }

    fn get<V, T: Asset>(map: &HashMap<Uuid, V>) -> &V {
        match map.get(&T::TYPE_UUID) {
            Some(value) => value,
            None => panic!("Asset {} is not registered", type_name::<T>()),
        }
    }
}
