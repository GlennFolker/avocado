use bevy_ecs::prelude::*;

#[derive(StageLabel)]
pub struct StartupStage;

#[derive(StageLabel)]
pub enum CoreStage {
    /// Time, input poll, and events update.
    SysUpdate,
    PreUpdate,
    Update,
    /// Global transform update.
    PostUpdate,
    SysPostUpdate,
}

#[derive(SystemLabel)]
pub enum CoreLabel {
    TimeUpdate,
    FixedTimeUpdate,
    EventUpdate,
    EntityValidation,

    ComputeTransform,
}
