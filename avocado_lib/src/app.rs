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

    pub fn default_runner() -> Box<dyn FnOnce(App) -> !> {
        Box::new(|mut app| {
            let exit = panic::catch_unwind(AssertUnwindSafe(|| app.schedule
                .as_mut().unwrap()
                .run(app.world.as_mut().unwrap())
            ));

            drop(app);
            process::exit(if exit.is_ok() {
                0
            } else {
                1
            });
        })
    }

    pub fn set_runner(&mut self, runner: impl FnOnce(&mut App) -> () + 'static) {
        let mut runner = Some(runner);
        self.runner = Some(Box::new(move |mut app| {
            let runner = runner.take().unwrap();
            let exit = panic::catch_unwind(AssertUnwindSafe(|| runner(&mut app)));

            drop(app);
            process::exit(if exit.is_ok() {
                0
            } else {
                1
            });
        }));
    }

    #[inline]
    pub fn set_runner_entire(&mut self, runner: impl FnOnce(App) -> ! + 'static) {
        self.runner = Some(Box::new(runner))
    }

    #[inline]
    pub fn run(&mut self) -> ! {
        let mut app = mem::replace(self, Self {
            world: None,
            schedule: None,
            runner: None,
        });

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
        let world = self.world_mut();
        if !world.contains_resource::<T>() {
            world.insert_non_send_resource(prov());
        }

        world.non_send_resource_mut::<T>()
    }
}

pub trait Subsystem {
    fn init(app: &mut App);
}
