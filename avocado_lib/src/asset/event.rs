use crate::incl::*;

pub enum AssetEvent<T: Asset> {
    Created(Handle<T>),
    Removed(Handle<T>),
}

pub struct AssetGraphDoneEvent;
