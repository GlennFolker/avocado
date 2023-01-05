use bevy_ecs::prelude::*;
use std::time::{
    Duration, Instant
};

#[derive(Resource, Default)]
pub struct Time {
    first_update: Option<Instant>,
    last_update: Option<Instant>,
    last_paused: Option<Instant>,

    total_paused: Duration,
    delta: Duration,
    elapsed: Duration,
    elapsed_no_pause: Duration,

    total_paused_sec: f64,
    delta_sec: f64,
    elapsed_sec: f64,
    elapsed_no_pause_sec: f64,

    pausing: bool,
    unpausing: bool,
}

impl Time {
    pub fn update_sys(mut time: ResMut<Time>) {
        time.update();
    }

    pub const fn secs(time: f64) -> u64 {
        time as u64
    }

    pub const fn nanos(time: f64) -> u32 {
        ((time % 1.) * 1_000_000_000.0) as u32
    }

    pub fn update(&mut self) {
        let now = Instant::now();

        if self.first_update.is_none() {
            self.first_update = Some(now);
        }

        if let Some(last_update) = self.last_update {
            self.delta = now.duration_since(last_update);
        } else {
            self.delta = Duration::new(0, 0);
        }

        self.last_update = Some(now);
        if self.pausing {
            self.last_paused = Some(now);
            self.pausing = false;
        } else if self.unpausing {
            self.last_paused = None;
            self.unpausing = false;
        }

        if let Some(last_paused) = self.last_paused {
            self.total_paused += now.duration_since(last_paused);
        }

        self.elapsed = now.duration_since(self.first_update.unwrap());
        self.elapsed_no_pause = self.elapsed - self.total_paused;

        self.total_paused_sec = self.total_paused.as_secs_f64();
        self.delta_sec = self.delta.as_secs_f64();
        self.elapsed_sec = self.elapsed.as_secs_f64();
        self.elapsed_no_pause_sec = self.elapsed_no_pause.as_secs_f64();
    }

    #[inline]
    pub fn is_paused(&self) -> bool {
        self.last_paused.is_some()
    }

    #[inline]
    pub fn pause(&mut self) {
        if self.unpausing {
            self.unpausing = false;
        } else {
            self.pausing = true;
        }
    }

    #[inline]
    pub fn unpause(&mut self) {
        if self.pausing {
            self.pausing = false;
        } else {
            self.unpausing = true;
        }
    }

    #[inline]
    pub fn first_update(&self) -> Option<Instant> {
        self.first_update
    }

    #[inline]
    pub fn last_update(&self) -> Option<Instant> {
        self.last_update
    }

    #[inline]
    pub fn last_paused(&self) -> Option<Instant> {
        self.last_paused
    }

    #[inline]
    pub fn total_paused(&self) -> Duration {
        self.total_paused
    }

    #[inline]
    pub fn delta(&self) -> Duration {
        self.delta
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    #[inline]
    pub fn elapsed_no_pause(&self) -> Duration {
        self.elapsed_no_pause
    }

    #[inline]
    pub fn total_paused_sec(&self) -> f64 {
        self.total_paused_sec
    }

    #[inline]
    pub fn delta_sec(&self) -> f64 {
        self.delta_sec
    }

    #[inline]
    pub fn elapsed_sec(&self) -> f64 {
        self.elapsed_sec
    }

    #[inline]
    pub fn elapsed_no_pause_sec(&self) -> f64 {
        self.elapsed_no_pause_sec
    }
}

#[derive(Resource, Default)]
pub struct FixedUpdate<const S: u64, const N: u32> {
    accum: Duration,
    accum_scl: f64,
    qualified: bool,
}

impl<const S: u64, const N: u32> FixedUpdate<S, N> {
    pub fn update_sys(time: Res<Time>, mut updater: ResMut<FixedUpdate<S, N>>) {
        updater.update(time.delta());
    }

    pub fn qualified_sys(updater: Res<FixedUpdate<S, N>>) -> bool {
        updater.qualified()
    }

    pub const fn duration() -> Duration {
        Duration::new(S, N)
    }

    pub fn update(&mut self, delta: Duration) {
        let duration = Self::duration();

        self.accum += delta;
        self.qualified = self.accum >= duration;
        if self.qualified {
            self.accum -= duration;
        }

        self.accum_scl = self.accum.div_duration_f64(duration);
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
