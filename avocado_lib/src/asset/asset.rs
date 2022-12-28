use crate::incl::*;

pub trait Asset: TypeUuid + AssetDyn {}
pub trait AssetDyn: 'static + TypeUuidDyn + Downcast + Debug + Send + Sync {}
impl_downcast!(AssetDyn);

impl<T: TypeUuid + AssetDyn> Asset for T {}
impl<T: 'static + TypeUuidDyn + Debug + Send + Sync> AssetDyn for T {}

pub(crate) trait AssetPipe: 'static + Downcast + Send + Sync {}
impl_downcast!(AssetPipe);

pub(crate) struct AssetChannel<T: Asset> {
    pub sender: Sender<AssetLife<T>>,
    pub receiver: Receiver<AssetLife<T>>,
}

impl<T: Asset> AssetPipe for AssetChannel<T> {}
impl<T: Asset> Default for AssetChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }
}

pub(crate) enum AssetLife<T: Asset> {
    Created(Cow<'static, Path>, T),
    Removed(Cow<'static, Path>),
}

pub enum AssetState {
    Unloaded,
    Loading,
    Loaded,
    Errored(String),
}

#[derive(Resource)]
pub struct Assets<T: Asset> {
    assets: HashMap<Cow<'static, Path>, Arc<T>>,
    counts: HashMap<Cow<'static, Path>, isize>,
    ref_change: Sender<RefChange<T>>,
}

impl<T: Asset> Assets<T> {
    pub fn get(&self, handle: &Handle<T>) -> Option<Arc<T>> {
        self.assets.get(&handle.handle_path).cloned()
    }

    pub fn add(&mut self, handle_path: Cow<'static, Path>, asset: T) -> Handle<T> {
        self.assets.insert(handle_path.clone(), Arc::new(asset));
        Handle::strong(handle_path, self.ref_change.clone())
    }

    pub(crate) fn new(ref_change: Sender<RefChange<T>>) -> Self {
        Self {
            assets: HashMap::default(),
            counts: HashMap::default(),
            ref_change,
        }
    }

    pub(crate) fn add_direct(&mut self, events: &mut EventWriter<AssetEvent<T>>, handle_path: Cow<'static, Path>, asset: T) {
        self.assets.insert(handle_path.clone(), Arc::new(asset));
        events.send(AssetEvent::Created(Handle::weak(handle_path)));
    }

    pub(crate) fn remove_direct(&mut self, events: &mut EventWriter<AssetEvent<T>>, handle_path: Cow<'static, Path>) {
        if self.assets.remove(&handle_path).is_some() {
            events.send(AssetEvent::Removed(Handle::weak(handle_path)));
        }
    }

    pub(crate) fn count(&self, handle_path: &Path) -> isize {
        *self.counts.get(handle_path).unwrap_or(&0)
    }

    pub(crate) fn incr_count(&mut self, handle_path: Cow<'static, Path>, incr: isize) {
        *self.counts.entry(handle_path).or_insert(0) += incr;
    }
}

pub enum AssetEvent<T: Asset> {
    Created(Handle<T>),
    Removed(Handle<T>),
}
