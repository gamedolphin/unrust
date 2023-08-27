use crate::types::DoRotate;
use rand::Rng;
use std::f32::consts::TAU;
use unrust::bevy::ecs as bevy_ecs;
use unrust::bevy::prelude::*;

#[derive(Default, Component)]
pub(crate) struct Clock {
    pub time_left: f32,
    pub toggle: bool,
}

pub(crate) fn init_rotation_timer(
    mut cubes: Query<(Entity, &DoRotate), Without<Clock>>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    for (entity, _) in &mut cubes {
        let mut comm = commands.entity(entity);
        comm.insert(Clock {
            time_left: 1.0 + rng.gen::<f32>() * 3.0,
            toggle: false,
        });
    }
}

pub(crate) fn enable_disable_rotation(mut cubes: Query<(&DoRotate, &mut Clock)>, timer: Res<Time>) {
    let mut rng = rand::thread_rng();
    for (_, mut clock) in &mut cubes {
        clock.time_left -= timer.delta_seconds();

        if clock.time_left > 0.0 {
            continue;
        }

        clock.time_left = 1.0 + rng.gen::<f32>() * 3.0;
        clock.toggle = !clock.toggle;
    }
}

pub(crate) fn rotate_cube(mut cubes: Query<(&mut Transform, &DoRotate, &Clock)>, timer: Res<Time>) {
    for (mut transform, cube, clock) in &mut cubes {
        if clock.toggle {
            continue;
        }

        transform.rotate_y(cube.speed * TAU * timer.delta_seconds());
    }
}
