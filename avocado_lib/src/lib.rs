#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(fn_traits)]
#![feature(never_type)]

use incl::*;

pub mod incl {
    pub use super::{
        app::*,
    };

    #[cfg(feature = "log")]
    pub use crate::log::*;

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

    pub use bevy_utils::{
        self, prelude::*,
        Entry, HashMap, HashSet,
    };

    pub use log;

    pub use std::{
        borrow::Cow,
        collections::VecDeque,
        io,
        mem,
        panic::{
            self,
            AssertUnwindSafe,
        },
        process,
    };
}

#[cfg(feature = "asset")]
pub mod asset;
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
    }
}
