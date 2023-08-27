use unrust::bevy::ecs as bevy_ecs;
use unrust::bevy::prelude::*;
use unrust::{unrust_setup, GamePlugin};

mod types;
use types::*;

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
    );
}
