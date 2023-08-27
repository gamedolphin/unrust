use unrust::bevy;
use unrust::bevy::ecs as bevy_ecs;
use unrust::{bevy_state, unity_authoring, unity_prefab};

#[unity_authoring]
pub struct DoRotate {
    pub speed: f32,
}

#[bevy_state]
pub enum GameState {
    #[default]
    HelloCubeSimple = 0,
    PrefabCube = 1,
    Parenting = 2,
    Enableable = 3,
    Boids = 4,
}

#[unity_prefab]
pub enum CubePrefabs {
    HelloCube,
}
