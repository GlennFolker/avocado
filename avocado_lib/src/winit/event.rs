use crate::incl::*;

#[derive(Deref, DerefMut)]
pub struct WindowResizedEvent(pub winit::PhysicalSize<u32>);
#[derive(Deref, DerefMut)]
pub struct WindowMovedEvent(pub winit::PhysicalPosition<i32>);

pub struct SuspendEvent;
pub struct ResumeEvent;
