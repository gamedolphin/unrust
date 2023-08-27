use std::f32::consts::TAU;

use crate::types::DoRotate;
use unrust::bevy::prelude::*;

pub(crate) fn rotate_cube(mut cubes: Query<(&mut Transform, &DoRotate)>, timer: Res<Time>) {
    for (mut transform, cube) in &mut cubes {
        // The speed is first multiplied by TAU which is a full rotation (360deg) in radians,
        // and then multiplied by delta_seconds which is the time that passed last frame.
        // In other words. Speed is equal to the amount of rotations per second.
        transform.rotate_y(cube.speed * TAU * timer.delta_seconds());
    }
}
