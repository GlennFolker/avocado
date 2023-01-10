use crate::core::Time;
use bevy_ecs::prelude::*;
use bevy_utils::default;
use std::{
    ops::{
        Deref, DerefMut,
    },
    time::Duration,
};

pub trait FixedUpdateWrap: Resource + Deref<Target = FixedUpdate> + DerefMut<Target = FixedUpdate> {
    fn new(world: &mut World, updater: FixedUpdate) -> Self;
}

#[derive(Debug, Default)]
pub struct FixedUpdate {
    duration: Duration,
    accum: Duration,
    accum_scl: f64,
    qualified: bool,
}

impl FixedUpdate {
    pub fn update_sys<T: FixedUpdateWrap>(time: Res<Time>, mut updater: ResMut<T>) {
        updater.update(time.delta());
    }

    pub fn qualified_sys<T: FixedUpdateWrap>(updater: Res<T>) -> bool {
        updater.qualified
    }

    pub fn new<T: FixedUpdateWrap>(world: &mut World, duration: Duration) -> T {
        T::new(world, Self {
            duration,
            accum: default(),
            accum_scl: default(),
            qualified: false,
        })
    }

    pub fn update(&mut self, delta: Duration) {
        self.accum += delta;
        self.qualified = self.accum >= self.duration;
        if self.qualified {
            self.accum -= self.duration;
        }

        self.accum_scl = self.accum.div_duration_f64(self.duration);
    }

    #[inline]
    pub fn accum(&self) -> Duration {
        self.accum
    }

    #[inline]
    pub fn accum_scl(&self) -> f32 {
        self.accum_scl as f32
    }

    #[inline]
    pub fn accum_scl_f64(&self) -> f64 {
        self.accum_scl
    }

    #[inline]
    pub fn qualified(&self) -> bool {
        self.qualified
    }
}
