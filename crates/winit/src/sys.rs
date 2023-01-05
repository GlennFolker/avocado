use avocado_core::prelude::*;

#[derive(StageLabel)]
pub enum RenderStage {
    Begin,
    Queue,
    Render,
    End,
}

#[derive(SystemLabel)]
pub enum RenderLabel {
    PrepareFrame,
    InitFrame,
    PresentFrame,

    ComputeGlobalCamera,

    Queue,
    Render,
}
