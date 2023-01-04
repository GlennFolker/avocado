use crate::incl::*;

#[derive(Deref, DerefMut)]
pub struct WindowResizedEvent(pub winit::PhysicalSize<u32>);
#[derive(Deref, DerefMut)]
pub struct WindowMovedEvent(pub winit::PhysicalPosition<i32>);

pub struct KeyboardEvent {
    /// `true` if pressed, `false` if released.
    pub pressed: bool,
    pub key: KeyCode,
}

pub struct KeyModifierEvent {
    pub alt: bool,
    pub ctrl: bool,
    pub logo: bool,
    pub shift: bool,
}

pub struct SuspendEvent;
pub struct ResumeEvent;
