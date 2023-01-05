#![feature(const_fn_floating_point_arithmetic)]
#![feature(div_duration)]
#![feature(never_type)]

use bevy_ecs::{
    prelude::*,
    schedule::ShouldRun,
};
use bevy_tasks::{
    TaskPoolBuilder,
    AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool,
};

mod app;
mod config;
mod event;
mod sys;
mod time;

pub use app::*;
pub use config::*;
pub use event::*;
pub use sys::*;
pub use time::*;

pub mod re_exports {
    pub use bevy_ecs;
    pub use bevy_tasks;
    pub use iyes_loopless;
}

pub mod prelude {
    pub use crate::{
        re_exports::*,
        CoreSubsystem,
        App, Subsystem,
        TaskPoolConfig, TaskPoolConf,
        ExitEvent, ExitReason,
        StartupStage, CoreStage, CoreLabel,
        Time, FixedUpdate,
    };

    pub use bevy_ecs::{
        prelude::*,
        entity::{
            EntityMap,
            MapEntities, MapEntitiesError,
        },
        event::Event,
        schedule::{
            ShouldRun,
            StageLabel, SystemLabel,
            SystemLabelId,
        },
        system::{
            lifetimeless::{
                SRes,
            },
            SystemState, BoxedSystem, FunctionSystem,
            SystemParam, ReadOnlySystemParamFetch, SystemParamItem,
            Command,
            EntityCommands,
        },
        query::{
            WorldQuery, ReadOnlyWorldQuery,
        },
    };
    pub use bevy_tasks::{
        prelude::*,
        Scope,
    };
    pub use iyes_loopless::prelude::*;
}

pub struct CoreSubsystem;
impl Subsystem for CoreSubsystem {
    fn init(app: &mut App) {
        let config = app.res_or(TaskPoolConfig::default);
        let create = |conf: &TaskPoolConf| {
            let mut builder = TaskPoolBuilder::new();
            builder = if let Some(num) = conf.threads {
                builder.num_threads(num)
            } else { builder };

            builder = if let Some(size) = conf.stack_size {
                builder.stack_size(size)
            } else { builder };

            builder = if let Some(ref name) = conf.thread_name {
                builder.thread_name(name.clone().into_owned())
            } else { builder };

            builder.build()
        };

        AsyncComputeTaskPool::init(|| create(&config.async_pool));
        ComputeTaskPool::init(|| create(&config.compute_pool));
        IoTaskPool::init(|| create(&config.io_pool));

        app
            .stage(CoreStage::SysUpdate, SystemStage::parallel()
                .with_system(Time::update_sys.label(CoreLabel::TimeUpdate))
            )

            .stage(StartupStage, SystemStage::parallel()
                .with_run_criteria(ShouldRun::once)
            )

            .stage(CoreStage::PreUpdate, SystemStage::parallel())
            .stage(CoreStage::Update, SystemStage::parallel())
            .stage(CoreStage::PostUpdate, SystemStage::parallel())
            .stage(CoreStage::SysPostUpdate, SystemStage::parallel()
                .with_system(bevy_tasks::tick_global_task_pools_on_main_thread.at_end())
            )

            .init_res::<Time>()
            .event::<ExitEvent>();
    }
}
