use crate::incl::*;

pub struct Handle<T: Asset> {
    pub(crate) handle_path: Cow<'static, Path>,
    pub(crate) handle_type: HandleType<T>,
}

impl<T: Asset> Handle<T> {
    pub(crate) fn strong(handle_path: Cow<'static, Path>, ref_change: Sender<RefChange<T>>) -> Self {
        if let Err(msg) = ref_change.send(RefChange::Incr(Self::weak(handle_path.clone()))) {
            log::warn!("Couldn't increment asset handle {:?}: {}", &handle_path, msg);
        }

        Self {
            handle_path: handle_path,
            handle_type: HandleType::Strong(ref_change),
        }
    }

    #[inline]
    pub fn weak(handle_path: Cow<'static, Path>) -> Self {
        Self {
            handle_path: handle_path,
            handle_type: HandleType::Weak,
        }
    }

    #[inline]
    pub fn clone_weak(&self) -> Self {
        Self {
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

impl<T: Asset> Drop for Handle<T> {
    fn drop(&mut self) {
        if let HandleType::Strong(ref sender) = self.handle_type {
            if let Err(msg) = sender.send(RefChange::Decr(self.clone_weak())) {
                log::warn!("Couldn't decrement asset handle {:?}: {}", &self.handle_path, msg);
            }
        }
    }
}

pub(crate) enum HandleType<T: Asset> {
    Weak,
    Strong(Sender<RefChange<T>>),
}

pub(crate) enum RefChange<T: Asset> {
    Incr(Handle<T>),
    Decr(Handle<T>),
}

pub(crate) trait RefPipe: 'static + Downcast + Debug + Send + Sync {}
impl_downcast!(RefPipe);

#[derive(Debug)]
pub(crate) struct RefChannel<T: Asset> {
    pub sender: Sender<RefChange<T>>,
    pub receiver: Receiver<RefChange<T>>,
}

impl<T: Asset> RefPipe for RefChannel<T> {}
impl<T: Asset> Default for RefChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver, }
    }
}
