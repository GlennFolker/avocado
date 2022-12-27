use crate::incl::*;

mod config;
mod event;
mod ext;
mod sys;
mod time;

pub use config::*;
pub use event::*;
pub use ext::AppExt as _;
pub use sys::*;
pub use time::*;

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
