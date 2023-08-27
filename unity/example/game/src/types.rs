use unrust::bevy;
use unrust::bevy::ecs as bevy_ecs;
use unrust::{bevy_state, unity_authoring, unity_prefab};

#[unity_authoring]
pub struct SampleComponent {
    pub speed: f32,
}