use crate::incl::*;

#[derive(Resource)]
pub struct AssetServer {
    reader: Arc<dyn AssetReader>,

    ref_channels: HashMap<Uuid, RefChannel>,
    asset_channels: HashMap<Uuid, Arc<dyn AssetPipe>>,

    states: HashMap<Uuid, Arc<RwLock<HashMap<Cow<'static, Path>, AssetState>>>>,
    loaders: HashMap<Uuid, Arc<dyn AssetLoader>>,
    changed: HashSet<Cow<'static, Path>>,

    load_syncs: Arc<RwLock<VecDeque<(Sender<()>, AssetLoadSyncCallback)>>>,
}

impl AssetServer {
    pub fn update_sys<T: Asset>(
        mut server: ResMut<AssetServer>,
        mut events: EventWriter<AssetEvent<T>>, mut assets: ResMut<Assets<T>>
    ) {
        server.update(&mut events, &mut assets);
    }

    pub fn post_update_sys(world: &mut World) {
        world.resource_scope(|world, mut server: Mut<AssetServer>| server.post_update(world));
    }

    pub fn new(reader: Arc<dyn AssetReader>) -> Self {
        Self {
            reader,

            ref_channels: HashMap::default(),
            asset_channels: HashMap::default(),

            states: HashMap::default(),
            loaders: HashMap::default(),
            changed: HashSet::default(),

            load_syncs: Arc::default(),
        }
    }

    pub fn state<T: Asset>(&self, handle: &Handle<T>) -> AssetState {
        Self::get::<_, T>(&self.states).read().get(handle.path()).unwrap_or(&AssetState::Unloaded).clone()
    }

    pub fn group_state<'a, T: Asset>(&self, handles: impl Iterator<Item = &'a Handle<T>>) -> AssetState {
        let mut state = AssetState::Loaded;
        for handle in handles {
            match Self::get::<_, T>(&self.states).read().get(handle.path()) {
                Some(current) => match current {
                    AssetState::Unloaded => {
                        if state == AssetState::Loaded {
                            state = AssetState::Unloaded;
                        }
                    },
                    AssetState::Loading => state = AssetState::Loading,
                    AssetState::Errored(ref msg) => return AssetState::Errored(msg.clone()),
                    AssetState::Loaded => {},
                },
                None => {
                    if state == AssetState::Loaded {
                        state = AssetState::Unloaded;
                    }
                },
            }
        }

        state
    }

    pub fn state_dyn(&self, handle: &HandleDyn) -> AssetState {
        Self::get_dyn(&self.states, &handle.uuid).read().get(handle.path()).unwrap_or(&AssetState::Unloaded).clone()
    }

    pub fn group_state_dyn<'a>(&self, handles: impl Iterator<Item = &'a HandleDyn>) -> AssetState {
        let mut state = AssetState::Loaded;
        for handle in handles {
            match Self::get_dyn(&self.states, &handle.uuid).read().get(handle.path()) {
                Some(current) => match current {
                    AssetState::Unloaded => {
                        if state == AssetState::Loaded {
                            state = AssetState::Unloaded;
                        }
                    },
                    AssetState::Loading => state = AssetState::Loading,
                    AssetState::Errored(ref msg) => return AssetState::Errored(msg.clone()),
                    AssetState::Loaded => {},
                },
                None => {
                    if state == AssetState::Loaded {
                        state = AssetState::Unloaded;
                    }
                },
            }
        }

        state
    }

    pub fn register<T: Asset>(&mut self) -> Assets<T> {
        let ref_channel = RefChannel::default();
        let ref_change = ref_channel.sender.clone();

        if self.ref_channels.insert(T::TYPE_UUID, ref_channel).is_some() {
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

    pub fn load<T: Asset>(&mut self, path: impl Into<Cow<'static, Path>>) -> Handle<T> {
        self.load_with(path, None::<NoAssetData>)
    }

    pub fn load_with<T: Asset>(
        &mut self,
        path: impl Into<Cow<'static, Path>>,
        data: Option<impl AssetData>
    ) -> Handle<T> {
        let path = path.into();
        let refs = &Self::get::<_, T>(&self.ref_channels).sender;
        let states = Self::get::<_, T>(&self.states);

        if {
            let mut should_load = false;

            let mut states = states.write();
            if !states.contains_key(&path) {
                states.insert(path.clone(), AssetState::Loading);
                should_load = true;
            }

            should_load
        } {
            let async_path = path.clone();
            let states = Arc::clone(&states);
            let reader = Arc::clone(&self.reader);
            let pipe = Arc::clone(Self::get::<_, T>(&self.asset_channels));

            let loader = self.get_loader::<T>();
            let data: Option<Box<dyn AssetData>> = match data {
                Some(data) => Some(Box::new(data)),
                None => None,
            };

            let load_syncs = Arc::clone(&self.load_syncs);
            IoTaskPool::get()
                .spawn(async move {
                    match loader.load(
                        reader, async_path.clone(),
                        data,
                        Box::new(move |callback| {
                            let (signal, shutdown) = crossbeam_channel::unbounded();
                            {
                                let mut load_syncs = load_syncs.write();
                                load_syncs.push_back((signal, callback));
                            }

                            shutdown.recv()?;
                            Ok(())
                        })
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

                            let mut states = states.write();
                            states.insert(async_path, AssetState::Errored(msg));
                        }
                    }
                })
                .detach();
        }

        Handle::strong(path, refs.clone())
    }

    pub fn update<T: Asset>(&mut self, events: &mut EventWriter<AssetEvent<T>>, assets: &mut Assets<T>) {
        let refs = &Self::get::<_, T>(&self.ref_channels).receiver;
        let pipe = Self::get::<_, T>(&self.asset_channels).downcast_ref::<AssetChannel<T>>().unwrap();

        loop {
            match refs.try_recv() {
                Ok(change) => match change {
                    RefChange::Incr(path) => {
                        assets.incr_count(path.clone(), 1);
                        self.changed.insert(path);
                    },
                    RefChange::Decr(path) => {
                        assets.incr_count(path.clone(), -1);
                        self.changed.insert(path);
                    },
                },
                Err(err) => match err {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => panic!("Ref channel {} disconnected", type_name::<T>()),
                },
            }
        }

        let mut state = Self::get::<_, T>(&self.states).write();
        for path in self.changed.drain() {
            if assets.count(&path) <= 0 {
                state.remove(&path);
                if let Err(msg) = pipe.sender.send(AssetLife::Removed(path.clone())) {
                    log::warn!("Couldn't send asset removal signal for {:?}: {}", &path, msg);
                }
            }
        }

        loop {
            match pipe.receiver.try_recv() {
                Ok(life) => match life {
                    AssetLife::Created(path, asset) => if assets.count(&path) > 0 {
                        state.insert(path.clone(), AssetState::Loaded);
                        assets.add_direct(events, path, asset);
                    },
                    AssetLife::Removed(path) => {
                        state.remove(&path); // Remove it again, just in case.
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

    pub fn post_update(&mut self, world: &mut World) {
        loop {
            let (signal, callback) = {
                let mut load_syncs = self.load_syncs.write();
                match load_syncs.pop_front() {
                    Some((signal, callback)) => (signal, callback),
                    None => break,
                }
            };

            callback(world);
            if let Err(msg) = signal.send(()) {
                log::warn!("Couldn't send continue signal: {}", msg);
            }
        }
    }

    fn get<V, T: Asset>(map: &HashMap<Uuid, V>) -> &V {
        match map.get(&T::TYPE_UUID) {
            Some(value) => value,
            None => panic!("Asset with type {} is not registered", type_name::<T>()),
        }
    }

    fn get_dyn<'a, V>(map: &'a HashMap<Uuid, V>, key: &Uuid) -> &'a V {
        match map.get(key) {
            Some(value) => value,
            None => panic!("Asset with UUID {} is not registered", key),
        }
    }
}
