#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#![feature(const_fn_floating_point_arithmetic)]
#![feature(div_duration)]
#![feature(fn_traits)]
#![feature(inline_const)]
#![feature(let_chains)]
#![feature(never_type)]

use incl::*;

pub mod incl {
    pub use super::{
        app::*,
    };

    #[cfg(feature = "asset")]
    pub use crate::asset::*;

    #[cfg(feature = "core")]
    pub use crate::core::*;

    #[cfg(feature = "log")]
    pub use crate::log::*;

    #[cfg(feature = "winit")]
    pub use crate::winit::*;

    pub use bevy_ecs::{
        self, prelude::*,
        entity::{
            EntityMap,
            MapEntities, MapEntitiesError,
        },
        event::Event,
        schedule::{
            ShouldRun,
            StageLabel, SystemLabel,
        },
        system::{
            Command,
            EntityCommands,
        },
        query::{
            WorldQuery, ReadOnlyWorldQuery,
        },
    };

    pub use bevy_reflect::{
        self, prelude::*,
        TypeUuid, TypeUuidDynamic as TypeUuidDyn,
    };

    pub use bevy_tasks::{
        self, prelude::*,
        TaskPool, TaskPoolBuilder,
    };

    pub use bevy_utils::{
        self, prelude::*,
        Entry, HashMap, HashSet,
        Uuid,
        Instant, Duration,
    };

    pub use iyes_loopless::{
        self, prelude::*,
    };

    pub use anyhow::{
        self,
        Context as _,
    };

    pub use cfg_if::cfg_if;
    pub use crossbeam_channel::{
        self,
        Sender, Receiver, TryRecvError,
    };

    pub use derive_more::{
        self,
        *,
    };

    pub use downcast_rs::{
        self,
        impl_downcast,
        Downcast, DowncastSync,
    };

    pub use log;
    pub use parking_lot::{
        self,
        RwLock,
    };

    pub use thiserror::Error;

    pub use std::{
        any::{
            type_name,
            Any,
        },
        borrow::Cow,
        collections::VecDeque,
        env,
        fmt::Debug,
        fs::{
            self,
            File
        },
        io::{
            self,
            Read, Write, Seek,
        },
        marker::PhantomData,
        mem,
        panic::{
            self,
            AssertUnwindSafe,
        },
        path::{
            Path, PathBuf,
        },
        process,
        sync::Arc,
    };
}

#[cfg(feature = "asset")]
pub mod asset;
#[cfg(feature = "core")]
pub mod core;
#[cfg(feature = "winit")]
pub mod winit;
#[cfg(feature = "log")]
pub mod log;

mod app;

pub struct AVocado;
impl Subsystem for AVocado {
    fn init(app: &mut App) {
        #[cfg(feature = "log")]
        app.init::<LogSubsystem>();

        #[cfg(feature = "core")]
        app.init::<CoreSubsystem>();

        #[cfg(feature = "asset")]
        app.init::<AssetSubsystem>();
    }
}
