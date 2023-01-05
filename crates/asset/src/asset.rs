use avocado_core::prelude::*;
use avocado_utils::prelude::*;

use crate::{
    Handle, RefChange,
};
use std::{
    borrow::Cow,
    fmt::Debug,
    path::Path,
};

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

#[derive(Debug, Clone, PartialEq)]
pub enum AssetState {
    Unloaded,
    Loading,
    Loaded,
    Errored(String),
}

#[derive(Resource)]
pub struct Assets<T: Asset> {
    assets: HashMap<Cow<'static, Path>, T>,
    counts: HashMap<Cow<'static, Path>, isize>,
    ref_change: Sender<RefChange>,
}

impl<T: Asset> Assets<T> {
    #[inline]
    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        self.assets.get(&handle.handle_path)
    }

    #[inline]
    pub fn add(&mut self, handle_path: Cow<'static, Path>, asset: T) -> Handle<T> {
        self.assets.insert(handle_path.clone(), asset);
        Handle::strong(handle_path, self.ref_change.clone())
    }

    /// Useful if you want to load a resource asynchronously and then take ownership of it, e.g. to store it
    /// as a `[bevy_ecs::Resource]`.
    #[inline]
    pub fn remove(&mut self, handle: Handle<T>) -> Result<T, Handle<T>> {
        if self.count(&handle.handle_path) > 1 || !self.assets.contains_key(&handle.handle_path) {
            Err(handle)
        } else {
            Ok(self.assets.remove(&handle.handle_path).unwrap())
        }
    }

    #[inline]
    pub(crate) fn new(ref_change: Sender<RefChange>) -> Self {
        Self {
            assets: HashMap::default(),
            counts: HashMap::default(),
            ref_change,
        }
    }

    #[inline]
    pub(crate) fn add_direct(&mut self, handle_path: Cow<'static, Path>, asset: T) {
        self.assets.insert(handle_path.clone(), asset);
    }

    #[inline]
    pub(crate) fn remove_direct(&mut self, handle_path: Cow<'static, Path>) {
        self.assets.remove(&handle_path);
    }

    #[inline]
    pub(crate) fn count(&self, handle_path: &Path) -> isize {
        *self.counts.get(handle_path).unwrap_or(&0)
    }

    #[inline]
    pub(crate) fn incr_count(&mut self, handle_path: Cow<'static, Path>, incr: isize) {
        *self.counts.entry(handle_path).or_insert(0) += incr;
    }
}
