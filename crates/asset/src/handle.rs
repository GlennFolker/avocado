use avocado_utils::prelude::*;

use crate::Asset;
use std::{
    borrow::Cow,
    hash::{
        Hash, Hasher,
    },
    marker::PhantomData,
    path::Path,
};

#[derive(Debug)]
pub struct Handle<T: Asset> {
    pub(crate) handle_path: Cow<'static, Path>,
    handle_type: HandleType,
    marker: PhantomData<T>,
}

impl<T: Asset> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.handle_path.hash(state);
    }
}

impl<T: Asset> Clone for Handle<T> {
    fn clone(&self) -> Self {
        let path = self.handle_path.clone();
        match self.handle_type {
            HandleType::Weak => Handle::weak(path),
            HandleType::Strong(ref sender) => Handle::strong(path, sender.clone()),
        }
    }
}

impl<T: Asset> Handle<T> {
    pub(crate) fn strong(handle_path: Cow<'static, Path>, ref_change: Sender<RefChange>) -> Self {
        if let Err(msg) = ref_change.send(RefChange::Incr(handle_path.clone())) {
            log::warn!("Couldn't increment asset handle {:?}: {}", &handle_path, msg);
        }

        Self {
            handle_path: handle_path,
            handle_type: HandleType::Strong(ref_change),
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn weak(handle_path: Cow<'static, Path>) -> Self {
        Self {
            handle_path: handle_path,
            handle_type: HandleType::Weak,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn clone_weak(&self) -> Self {
        Self {
            handle_path: self.handle_path.clone(),
            handle_type: HandleType::Weak,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn is_weak(&self) -> bool {
        if let HandleType::Weak = self.handle_type {
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn is_strong(&self) -> bool {
        !self.is_weak()
    }

    #[inline]
    pub fn as_dyn(self) -> HandleDyn {
        if let HandleType::Strong(ref sender) = &self.handle_type {
            if let Err(msg) = sender.send(RefChange::Incr(self.handle_path.clone())) {
                log::warn!("Couldn't decrement asset handle {:?}: {}", &self.handle_path, msg);
            }
        }

        HandleDyn {
            uuid: T::TYPE_UUID,
            handle_path: self.handle_path.clone(),
            handle_type: self.handle_type.clone(),
        }
    }

    #[inline]
    pub fn path(&self) -> &Path {
        &self.handle_path
    }
}

impl<T: Asset> Drop for Handle<T> {
    fn drop(&mut self) {
        if let HandleType::Strong(ref sender) = self.handle_type {
            if let Err(msg) = sender.send(RefChange::Decr(self.handle_path.clone())) {
                log::warn!("Couldn't decrement asset handle {:?}: {}", &self.handle_path, msg);
            }
        }
    }
}

#[derive(Debug, Error)]
#[error("Couldn't cast {} to the concrete type", &0)]
pub struct HandleCastError(pub HandleDyn);
pub type HandleCastResult<T> = Result<Handle<T>, HandleCastError>;

#[derive(Debug)]
pub struct HandleDyn {
    pub(crate) uuid: Uuid,
    pub(crate) handle_path: Cow<'static, Path>,
    handle_type: HandleType,
}

impl Clone for HandleDyn {
    fn clone(&self) -> Self {
        let path = self.handle_path.clone();
        match self.handle_type {
            HandleType::Weak => Self {
                uuid: self.uuid,
                handle_path: path,
                handle_type: HandleType::Weak,
            },
            HandleType::Strong(ref sender) => {
                if let Err(msg) = sender.send(RefChange::Incr(path.clone())) {
                    log::warn!("Couldn't increment asset handle {:?}: {}", &path, msg);
                }

                Self {
                    uuid: self.uuid,
                    handle_path: path,
                    handle_type: HandleType::Strong(sender.clone()),
                }
            }
        }
    }
}

impl HandleDyn {
    pub fn typed<T: Asset>(self) -> HandleCastResult<T> {
        let path = self.handle_path.clone();
        if &self.uuid == &T::TYPE_UUID {
            Ok(match &self.handle_type {
                HandleType::Weak => Handle::weak(path),
                HandleType::Strong(ref sender) => Handle::strong(path, sender.clone()),
            })
        } else {
            Err(HandleCastError(self))
        }
    }

    #[inline]
    pub fn clone_typed<T: Asset>(&self) -> HandleCastResult<T> {
        self.clone().typed::<T>()
    }

    #[inline]
    pub fn clone_weak_typed<T: Asset>(&self) -> HandleCastResult<T> {
        self.clone_weak().typed::<T>()
    }

    #[inline]
    pub fn clone_weak(&self) -> Self {
        Self {
            uuid: self.uuid,
            handle_path: self.handle_path.clone(),
            handle_type: HandleType::Weak,
        }
    }

    #[inline]
    pub fn is_weak(&self) -> bool {
        if let HandleType::Weak = self.handle_type {
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn is_strong(&self) -> bool {
        !self.is_weak()
    }

    #[inline]
    pub fn path(&self) -> &Path {
        &self.handle_path
    }
}

impl Drop for HandleDyn {
    fn drop(&mut self) {
        if let HandleType::Strong(ref sender) = self.handle_type {
            if let Err(msg) = sender.send(RefChange::Decr(self.handle_path.clone())) {
                log::warn!("Couldn't decrement asset handle {:?}: {}", &self.handle_path, msg);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum HandleType {
    Weak,
    Strong(Sender<RefChange>),
}

#[derive(Debug)]
pub(crate) enum RefChange {
    Incr(Cow<'static, Path>),
    Decr(Cow<'static, Path>),
}

#[derive(Debug)]
pub(crate) struct RefChannel {
    pub sender: Sender<RefChange>,
    pub receiver: Receiver<RefChange>,
}

impl Default for RefChannel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver, }
    }
}
