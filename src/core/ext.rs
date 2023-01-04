use crate::incl::*;

pub trait AppExt {
    fn headless_runner() -> Box<dyn FnOnce(App) -> !>;

    fn event<T: Event>(&mut self) -> &mut Self;

    fn fixed_timestep<const S: u64, const N: u32>(
        &mut self,
        after: impl StageLabel, label: impl StageLabel, stage: impl Stage
    ) -> &mut Self;

    fn startup_sys<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) -> &mut Self;

    fn exit_handle(&mut self, var: Arc<RwLock<Option<ExitReason>>>) -> &mut Self;
}

impl AppExt for App {
    fn headless_runner() -> Box<dyn FnOnce(App) -> !> {
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
    fn event<T: Event>(&mut self) -> &mut Self {
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
    fn fixed_timestep<const S: u64, const N: u32>(
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
    fn startup_sys<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) -> &mut Self {
        self.schedule_mut().add_system_to_stage(StartupStage, system);
        self
    }

    #[inline]
    fn exit_handle(&mut self, var: Arc<RwLock<Option<ExitReason>>>) -> &mut Self {
        self.sys(CoreStage::SysPostUpdate, move |mut exit_event: EventReader<ExitEvent>| {
            if !exit_event.is_empty() {
                let event = exit_event.iter().next_back().unwrap();
                *var.write() = Some(event.reason.clone());

                exit_event.clear();
            }
        })
    }
}
