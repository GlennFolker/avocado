use derive_more::*;
use winit::dpi::{
    PhysicalSize, PhysicalPosition,
};

#[derive(Deref, DerefMut)]
pub struct WindowResizedEvent(pub PhysicalSize<u32>);
#[derive(Deref, DerefMut)]
pub struct WindowMovedEvent(pub PhysicalPosition<i32>);

pub struct SuspendEvent;
pub struct ResumeEvent;
