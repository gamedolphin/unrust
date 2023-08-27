use bevy::{ecs::world::EntityMut, prelude::*};

use crate::PrefabData;

pub trait GamePlugin {
    fn initialize(&self, app: &mut App);
    fn register(&self, world: &mut World, prefabs: PrefabData);

    #[allow(clippy::missing_safety_doc)]
    unsafe fn spawn_custom(
        &self,
        entity: &mut EntityMut,
        custom: *const u8,
        custom_len: usize,
        custom_state: *const u8,
        custom_state_len: usize,
    );
}
