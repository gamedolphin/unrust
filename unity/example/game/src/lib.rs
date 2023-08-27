use unrust::bevy::ecs as bevy_ecs;
use unrust::bevy::prelude::*;
use unrust::{unrust_setup, GamePlugin};

mod types;
use types::*;

mod hello_cube_enableable;
mod hello_cube_parenting;
mod hello_cube_simple;

#[unrust_setup((
    (DoRotate,Boid,),
    (GameState,),
    (CubePrefabs,BoidPrefabs,),
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
    );
}
