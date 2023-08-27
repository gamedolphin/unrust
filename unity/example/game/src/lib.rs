use bevy_common_assets::toml::TomlAssetPlugin;
use unrust::bevy::ecs as bevy_ecs;
use unrust::bevy::prelude::*;
use unrust::{unrust_setup, GamePlugin};

mod types;
use types::*;

mod hello_cube_enableable;
mod hello_cube_parenting;
mod hello_cube_prefab;
mod hello_cube_simple;

#[unrust_setup((
    (DoRotate,),
    (GameState,),
    (CubePrefabs,),
))]
pub fn setup(app: &mut App) {
    app.add_systems(
        Update,
        hello_cube_simple::rotate_cube.run_if(in_state(types::GameState::HelloCubeSimple)),
    )
    .add_systems(
        Update,
        (
            hello_cube_enableable::init_rotation_timer,
            hello_cube_enableable::rotate_cube,
            hello_cube_enableable::enable_disable_rotation,
        )
            .run_if(in_state(types::GameState::Enableable)),
    )
    .add_systems(
        Update,
        (
            hello_cube_parenting::track_old_parent,
            hello_cube_parenting::parent_reparent,
            hello_cube_simple::rotate_cube,
        )
            .run_if(in_state(types::GameState::Parenting)),
    )
    .add_plugins(
        TomlAssetPlugin::<hello_cube_prefab::CubePrefabSpawnSettings>::new(&["spawn.toml"]),
    )
    .add_systems(Startup, hello_cube_prefab::load_spawn_cube_settings)
    .add_systems(
        Update,
        (
            hello_cube_prefab::spawn_cube,
            hello_cube_simple::rotate_cube,
            hello_cube_prefab::fall_system,
        )
            .run_if(in_state(types::GameState::PrefabCube)),
    );
}
