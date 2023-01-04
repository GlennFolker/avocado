mod key;

pub use key::*;

use crate::incl::*;

pub trait InputAction: PartialEq + Eq + Hash {}
impl<T: PartialEq + Eq + Hash> InputAction for T {}

#[derive(Debug, Copy, Clone)]
pub enum InputValue {
    Single {
        pressed: bool,
        just_pressed: bool,
    },
    Axis(f32),
    Axes(Vec2),
}

#[derive(Component)]
pub struct InputState<T: InputAction> {
    pub(crate) states: HashMap<T, InputValue>,
}

impl<T: InputAction> InputState<T> {
    #[inline]
    pub fn pressed(&self, act: T) -> bool {
        let InputValue::Single { pressed, .. } = self.states[&act] else {
            panic!("`pressed()` is only for single input values");
        };
        pressed
    }

    #[inline]
    pub fn just_pressed(&self, act: T) -> bool {
        let InputValue::Single { just_pressed, .. } = self.states[&act] else {
            panic!("`just_pressed()` is only for single input values");
        };
        just_pressed
    }

    #[inline]
    pub fn axis(&self, act: T) -> f32 {
        let InputValue::Axis(axis) = self.states[&act] else {
            panic!("`axis()` is only for axis input values");
        };
        axis
    }

    #[inline]
    pub fn axes(&self, act: T) -> Vec2 {
        let InputValue::Axes(axes) = self.states[&act] else {
            panic!("`axes()` is only for axes input values");
        };
        axes
    }
}

#[derive(Resource)]
pub struct InputManager<T: InputAction> {
    marker: PhantomData<T>,
}

impl<T: InputAction> InputManager<T> {
    pub fn update_sys() {
        
    }
}

/*pub struct InputMap<T: InputAction> {

}*/
