use crate::types::DoRotate;
use unrust::bevy::ecs as bevy_ecs;
use unrust::bevy::prelude::*;

#[derive(Default, Component)]
pub struct Clock {
    pub time_left: f32,
    pub toggle: bool,
}

#[derive(Component)]
pub struct OldParent {
    pub parent: Entity,
}

pub(crate) fn track_old_parent(
    mut cubes: Query<(Entity, &DoRotate, &Parent), Without<OldParent>>,
    mut commands: Commands,
) {
    for (entity, _, up) in &mut cubes {
        let mut comm = commands.entity(entity);
        comm.insert(OldParent { parent: up.get() });
    }
}

pub(crate) fn parent_reparent(
    mut cubes: Query<(Entity, &DoRotate, &OldParent)>,
    mut commands: Commands,
    timer: Res<Time>,
    mut clock: Local<Clock>,
) {
    clock.time_left -= timer.delta_seconds();

    if clock.time_left > 0.0 {
        return;
    }

    clock.time_left = 2.0;
    clock.toggle = !clock.toggle;

    for (entity, _, op) in &mut cubes {
        let mut comm = commands.entity(entity);
        if clock.toggle {
            comm.remove_parent_in_place();
        } else {
            comm.set_parent_in_place(op.parent);
        }
    }
}
