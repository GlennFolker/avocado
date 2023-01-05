use avocado_input::prelude::*;
use avocado_utils::prelude::*;

use winit::dpi::{
    PhysicalSize, PhysicalPosition,
};

#[derive(Deref, DerefMut)]
pub struct WindowResizedEvent(pub PhysicalSize<u32>);
#[derive(Deref, DerefMut)]
pub struct WindowMovedEvent(pub PhysicalPosition<i32>);

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
