use crate::core::prelude::*;
use std::marker::PhantomData;

mod act;
mod binding;
mod event;
mod key;
mod manager;

pub use act::*;
pub use binding::*;
pub use event::*;
pub use key::*;
pub use manager::*;

pub mod prelude {
    pub use crate::input::{
        InputSubsystem,
        KeyCode,
        InputAction, InputValue, InputState,
        InputBinding, InputLinear, InputAxis, InputBindings,
        KeyEvent, KeyModifierEvent,
        InputManager,
    };
}

pub struct InputSubsystem<T: InputAction> {
    _marker: PhantomData<T>,
}

impl<T: InputAction> Subsystem for InputSubsystem<T> {
    fn init(app: &mut App) {
        app
            .init_res::<InputManager<T>>()
            .sys(CoreStage::SysUpdate, InputManager::<T>::update_sys);
    }
}
