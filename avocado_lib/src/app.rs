use crate::incl::*;

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
            let (world, schedule) = app.unzip_mut();
            let crashed = panic::catch_unwind(AssertUnwindSafe(|| schedule.run(world))).is_err();

            drop(app);
            process::exit(if crashed {
                1
            } else {
                0
            });
        })
    }

    pub fn headless_runner() -> Box<dyn FnOnce(App) -> !> {
        Box::new(|mut app| {
            let (world, schedule) = app.unzip_mut();
            let crashed = panic::catch_unwind(AssertUnwindSafe(|| loop {
                schedule.run(world)
            })).is_err();

            drop(app);
            process::exit(if crashed {
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
    pub fn sys<Params>(&mut self, label: impl StageLabel, system: impl IntoSystemDescriptor<Params>) -> &mut Self {
        self.schedule_mut().add_system_to_stage(label, system);
        self
    }
}

pub trait Subsystem {
    fn init(app: &mut App);
}
