use crate::incl::*;

pub trait AppExt {
    fn event<T: Event>(&mut self) -> &mut Self;

    fn fixed_timestep<const S: u64, const N: u32>(
        &mut self,
        after: impl StageLabel, label: impl StageLabel, stage: impl Stage
    ) -> &mut Self;

    fn startup_sys<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) -> &mut Self;
}

impl AppExt for App {
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
}
