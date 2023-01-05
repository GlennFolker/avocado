use avocado_utils::re_exports::*;

use crate::{
    StartupStage, CoreStage, CoreLabel,
    ExitEvent, ExitReason,
    FixedUpdate,
};

use bevy_ecs::{
    prelude::*,
    event::Event,
};
use parking_lot::RwLock;
use std::{
    mem,
    panic::{
        self,
        AssertUnwindSafe,
    },
    process,
    sync::Arc,
};

pub struct App {
    world: Option<World>,
    schedule: Option<Schedule>,
    runner: Option<Box<dyn FnOnce(App) -> !>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            world: Some(World::new()),
            schedule: Some(Schedule::default()),
            runner: Some(Self::default_runner()),
        }
    }

    /// Creates an empty, unusable `App` instance. Only use this to deallocate the previous instance.
    pub fn empty() -> Self {
        Self {
            world: None,
            schedule: None,
            runner: None,
        }
    }

    pub fn default_runner() -> Box<dyn FnOnce(App) -> !> {
        Box::new(|mut app| {
            let exit = Arc::default();
            app.exit_handle(Arc::clone(&exit));

            let (world, schedule) = app.unzip_mut();
            let result = panic::catch_unwind(AssertUnwindSafe(|| loop {
                schedule.run(world);

                let exit = exit.read();
                match &*exit {
                    Some(ExitReason::Graceful) => break,
                    Some(ExitReason::Error(msg)) => panic!("{}", &msg),
                    None => {},
                }
            }));

            if let Err(ref err) = result {
                log::error!("App crashed: {:?}", err);
            } else {
                log::info!("App exited gracefully");
            }

            drop(app);
            process::exit(if result.is_err() {
                1
            } else {
                0
            });
        })
    }

    #[inline]
    pub fn set_runner(&mut self, runner: impl FnOnce(App) -> ! + 'static) -> &mut Self {
        self.runner = Some(Box::new(runner));
        self
    }

    #[inline]
    pub fn unzip(&self) -> (&World, &Schedule) {
        (self.world.as_ref().unwrap(), self.schedule.as_ref().unwrap())
    }

    #[inline]
    pub fn unzip_mut(&mut self) -> (&mut World, &mut Schedule) {
        (self.world.as_mut().unwrap(), self.schedule.as_mut().unwrap())
    }

    #[inline]
    pub fn run(&mut self) -> ! {
        let mut app = mem::replace(self, Self::empty());

        let runner = app.runner.take().unwrap();
        runner(app);
    }

    #[inline]
    pub fn init<T: Subsystem>(&mut self) -> &mut Self {
        T::init(self);
        self
    }

    #[inline]
    pub fn world(&self) -> &World {
        self.world.as_ref().unwrap()
    }

    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        self.world.as_mut().unwrap()
    }

    #[inline]
    pub fn schedule(&self) -> &Schedule {
        self.schedule.as_ref().unwrap()
    }

    #[inline]
    pub fn schedule_mut(&mut self) -> &mut Schedule {
        self.schedule.as_mut().unwrap()
    }

    #[inline]
    pub fn init_res<T: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world_mut().init_resource::<T>();
        self
    }

    #[inline]
    pub fn insert_res<T: Resource>(&mut self, res: T) -> &mut Self {
        self.world_mut().insert_resource(res);
        self
    }

    #[inline]
    pub fn remove_res<T: Resource>(&mut self) -> Option<T> {
        self.world_mut().remove_resource::<T>()
    }

    #[inline]
    pub fn res<T: Resource>(&self) -> Option<&T> {
        self.world().get_resource::<T>()
    }

    #[inline]
    pub fn res_mut<T: Resource>(&mut self) -> Option<Mut<'_, T>> {
        self.world_mut().get_resource_mut::<T>()
    }

    #[inline]
    pub fn res_or<T: Resource>(&mut self, prov: impl FnOnce() -> T) -> Mut<'_, T> {
        self.world_mut().get_resource_or_insert_with(prov)
    }

    #[inline]
    pub fn init_res_ns<T: 'static + FromWorld>(&mut self) -> &mut Self {
        self.world_mut().init_non_send_resource::<T>();
        self
    }

    #[inline]
    pub fn insert_res_ns<T: 'static>(&mut self, res: T) -> &mut Self {
        self.world_mut().insert_non_send_resource(res);
        self
    }

    #[inline]
    pub fn remove_res_ns<T: 'static>(&mut self) -> Option<T> {
        self.world_mut().remove_non_send_resource::<T>()
    }

    #[inline]
    pub fn res_ns<T: 'static>(&self) -> Option<&T> {
        self.world().get_non_send_resource::<T>()
    }

    #[inline]
    pub fn res_ns_mut<T: 'static>(&mut self) -> Option<Mut<'_, T>> {
        self.world_mut().get_non_send_resource_mut::<T>()
    }

    #[inline]
    pub fn res_ns_or<T: 'static>(&mut self, prov: impl FnOnce() -> T) -> Mut<'_, T> {
        if !self.has_res::<T>() {
            self.insert_res_ns(prov());
        }

        self.res_ns_mut::<T>().unwrap()
    }

    #[inline]
    pub fn has_res<T: 'static>(&self) -> bool {
        self.world().contains_resource::<T>()
    }

    #[inline]
    pub fn event<T: Event>(&mut self) -> &mut Self {
        if !self.has_res::<Events<T>>() {
            self
                .init_res::<Events<T>>()
                .sys(CoreStage::SysUpdate, Events::<T>::update_system
                    .label(CoreLabel::EventUpdate)
                );
        }

        self
    }

    #[inline]
    pub fn fixed_timestep<const S: u64, const N: u32>(
        &mut self,
        after: impl StageLabel, label: impl StageLabel, stage: impl Stage
    ) -> &mut Self {
        self.stage_after(after, label, stage);
        if !self.has_res::<FixedUpdate<S, N>>() {
            self.init_res::<FixedUpdate<S, N>>()
                .sys(CoreStage::SysUpdate, FixedUpdate::<S, N>::update_sys
                    .label(CoreLabel::FixedTimeUpdate)
                    .after(CoreLabel::TimeUpdate)
                );
        }

        self
    }

    #[inline]
    pub fn stage(&mut self, label: impl StageLabel, stage: impl Stage) -> &mut Self {
        self.schedule_mut().add_stage(label, stage);
        self
    }

    #[inline]
    pub fn stage_after(&mut self, after: impl StageLabel, label: impl StageLabel, stage: impl Stage) -> &mut Self {
        self.schedule_mut().add_stage_after(after, label, stage);
        self
    }

    #[inline]
    pub fn stage_before(&mut self, before: impl StageLabel, label: impl StageLabel, stage: impl Stage) -> &mut Self {
        self.schedule_mut().add_stage_before(before, label, stage);
        self
    }

    #[inline]
    pub fn sys<Params>(&mut self, label: impl StageLabel, system: impl IntoSystemDescriptor<Params>) -> &mut Self {
        self.schedule_mut().add_system_to_stage(label, system);
        self
    }

    #[inline]
    pub fn startup_sys<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) -> &mut Self {
        self.schedule_mut().add_system_to_stage(StartupStage, system);
        self
    }

    #[inline]
    pub fn exit_handle(&mut self, var: Arc<RwLock<Option<ExitReason>>>) -> &mut Self {
        self.sys(CoreStage::SysPostUpdate, move |mut exit_event: EventReader<ExitEvent>| {
            if !exit_event.is_empty() {
                let event = exit_event.iter().next_back().unwrap();
                *var.write() = Some(event.reason.clone());

                exit_event.clear();
            }
        })
    }
}

pub trait Subsystem {
    fn init(app: &mut App);
}
