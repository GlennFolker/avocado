use crate::incl::*;

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
}
