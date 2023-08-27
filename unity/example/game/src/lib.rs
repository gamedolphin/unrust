use unrust::bevy::ecs as bevy_ecs;
use unrust::bevy::prelude::*;
use unrust::{unrust_setup, GamePlugin};

mod types;
use types::*;

#[unrust_setup((
    (SampleComponent,),
    (),
    (),
))]
pub fn setup(app: &mut App) {}
