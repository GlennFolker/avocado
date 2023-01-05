mod bin_pack;

pub use bin_pack::*;

pub mod re_exports {
    pub use ::bevy_math;
    pub use ::bevy_reflect;
    pub use ::bevy_utils;

    pub use ::anyhow;
    pub use ::async_channel;
    pub use ::cfg_if;
    pub use ::crossbeam_channel;
    pub use ::derive_more;
    pub use ::downcast_rs;
    pub use ::futures_lite;
    pub use ::parking_lot;
    pub use ::smallvec;
    pub use ::thiserror;
}

pub mod prelude {
    pub use crate::{
        re_exports::*,
        BinPack, BinPackResult,
    };
    pub use bevy_math::prelude::*;
    pub use bevy_reflect::{
        prelude::*,
        Uuid,
        TypeUuid, TypeUuidDynamic as TypeUuidDyn,
    };
    pub use bevy_utils::{
        BoxedFuture,
        HashMap, HashSet, Entry,
        default,
    };
    pub use async_channel::{
        Receiver as ReceiverAsync,
        Sender as SenderAsync,
    };
    pub use crossbeam_channel::{
        Receiver, Sender,
    };
    pub use derive_more::*;
    pub use downcast_rs::{
        impl_downcast,
        Downcast, DowncastSync,
    };
    pub use futures_lite::future;
    pub use parking_lot::RwLock;
    pub use smallvec::SmallVec;
    pub use thiserror::Error;
}
