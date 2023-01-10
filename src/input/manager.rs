use crate::{
    core::prelude::*,
    input::{
        KeyCode,
        InputAction, InputValue, InputState,
        InputBindings,
        KeyEvent,
    },
};
use bevy_utils::{
    default,
    HashMap, HashSet,
};
use smallvec::SmallVec;
use std::cell::Cell;

#[derive(Resource)]
pub struct InputManager<T: InputAction> {
    key_down: HashSet<KeyCode>,
    key_tapped: HashSet<KeyCode>,

    key_changed: HashSet<KeyCode>,
    act_changed: HashSet<T>,

    key_assoc: HashMap<KeyCode, SmallVec<[T; 4]>>,
    values: HashMap<T, InputValue>,
}

impl<T: InputAction> InputManager<T> {
    pub fn update_sys(
        mut manager: ResMut<Self>, bindings: Res<InputBindings<T>>,
        mut key_events: EventReader<KeyEvent>,
        mut query: Query<&mut InputState<T>>,
    ) {
        let mut changed = false;
        let mut updated = false;

        if !manager.key_tapped.is_empty() {
            changed = true;
            manager.key_tapped.clear();
        }

        manager.key_changed.clear();
        for event in key_events.iter() {
            manager.key_changed.insert(event.key);
            if event.pressed {
                if manager.key_down.insert(event.key) {
                    manager.key_tapped.insert(event.key);
                }
            } else {
                manager.key_down.remove(&event.key);
            }
        }

        if bindings.is_changed() {
            changed = true;
            updated = true;

            manager.values.drain_filter(|key, _| !bindings.map.contains_key(key));
            manager.key_assoc.clear();

            for (act, bind) in &bindings.map {
                for key in bind.keys() {
                    manager.key_assoc.entry(key).or_insert_with(default).push(*act);
                }

                let value = bindings.map[act].value(&manager.key_down, &manager.key_tapped);
                manager.values.insert(*act, value);
            }
        }

        if changed {
            if !updated {
                // Safety: There is guaranteed to be no data races whatsoever.
                let act_changed = unsafe { &mut *(&mut manager.act_changed as *mut HashSet<T> as *mut Cell<HashSet<T>>) };

                act_changed.get_mut().clear();
                for changed in &manager.key_changed {
                    let Some(assoc) = manager.key_assoc.get(changed) else { continue };
                    for key in assoc {
                        act_changed.get_mut().insert(*key);
                    }
                }

                for changed in act_changed.get_mut().iter() {
                    let Some(bind) = bindings.map.get(changed) else { continue };

                    let value = bind.value(&manager.key_down, &manager.key_tapped);
                    manager.values.insert(*changed, value);
                }
            }

            for mut state in &mut query {
                state.values = manager.values.clone();
            }
        }
    }
}

impl<T: InputAction> FromWorld for InputManager<T> {
    fn from_world(world: &mut World) -> Self {
        if !world.contains_resource::<InputBindings<T>>() {
            world.init_resource::<InputBindings<T>>();
        }

        Self {
            key_down: HashSet::default(),
            key_tapped: HashSet::default(),

            key_changed: HashSet::default(),
            act_changed: HashSet::default(),

            key_assoc: HashMap::default(),
            values: HashMap::default(),
        }
    }
}
