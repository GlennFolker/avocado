use crate::incl::*;

#[derive(StageLabel)]
pub struct StartupStage;

#[derive(StageLabel)]
pub enum CoreStage {
    /// Time and events update.
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
