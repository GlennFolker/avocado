#![feature(const_fn_floating_point_arithmetic)]
#![feature(div_duration)]
#![feature(let_chains)]
#![feature(never_type)]

#[cfg(feature = "asset")]
pub mod asset;
#[cfg(feature = "core")]
pub mod core;
#[cfg(feature = "g2d")]
pub mod g2d;
#[cfg(feature = "graphics")]
pub mod graphics;
#[cfg(feature = "input")]
pub mod input;
#[cfg(feature = "log")]
pub mod log;
#[cfg(feature = "winit")]
pub mod winit;

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

    #[cfg(feature = "core")]
    pub use crate::core::re_exports::*;
    #[cfg(feature = "graphics")]
    pub use crate::graphics::re_exports::*;
    #[cfg(feature = "log")]
    pub use crate::log::re_exports::*;
    #[cfg(feature = "winit")]
    pub use crate::winit::re_exports::*;
}

pub mod prelude {
    pub use crate::{
        re_exports::*,
        AVocado,
    };

    #[cfg(feature = "asset")]
    pub use crate::asset::prelude::*;
    #[cfg(feature = "core")]
    #[cfg(feature = "g2d")]
    pub use crate::g2d::prelude::*;
    pub use crate::core::prelude::*;
    #[cfg(feature = "graphics")]
    pub use crate::graphics::prelude::*;
    #[cfg(feature = "input")]
    pub use crate::input::prelude::*;
    #[cfg(feature = "log")]
    pub use crate::log::prelude::*;
    #[cfg(feature = "winit")]
    pub use crate::winit::prelude::*;

    pub use bevy_math::prelude::*;
    pub use bevy_reflect::{
        prelude::*,
        TypeUuid,
    };
    pub use bevy_tasks::{
        prelude::*,
        Scope,
    };
    pub use bevy_utils::{
        prelude::*,
        Entry, HashMap, HashSet,
        Uuid,
        Instant, Duration,
    };
    pub use derive_more::*;
}

pub struct AVocado;

#[cfg(feature = "core")]
impl core::Subsystem for AVocado {
    fn init(app: &mut core::App) {
        #[cfg(feature = "log")]
        app.init::<log::LogSubsystem>();

        app.init::<core::CoreSubsystem>();

        #[cfg(feature = "asset")]
        app.init::<asset::AssetSubsystem>();
        #[cfg(feature = "graphics")]
        app.init::<graphics::GraphicsSubsystem>();
        #[cfg(feature = "winit")]
        app.init::<winit::WinitSubsystem>();
        #[cfg(feature = "g2d")]
        app.init::<g2d::G2dSubsystem>();
    }
}
