use rand::Rng;
use serde::Deserialize;
use unrust::bevy::ecs as bevy_ecs;
use unrust::bevy::reflect as bevy_reflect;
use unrust::bevy::reflect::{TypePath, TypeUuid};
use unrust::tracing;
use unrust::{bevy::prelude::*, UnityEntity};

use crate::types::{CubePrefabsResource, DoRotate};

#[derive(Default)]
pub(crate) struct Spawned {
    pub done: bool,
}

#[derive(Deserialize, TypePath, TypeUuid)]
#[uuid = "513be529-bfeb-41b3-9db0-4b8b380a2c46"]
pub struct CubePrefabSpawnSettings {
    pub spawn_count: i32,
}

#[derive(Resource)]
pub struct CubePrefabSpawnSettingsAsset(pub Handle<CubePrefabSpawnSettings>);

pub(crate) fn load_spawn_cube_settings(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = CubePrefabSpawnSettingsAsset(asset_server.load("Resources/cube.spawn.toml"));
    commands.insert_resource(handle);
}

pub(crate) fn spawn_cube(
    prefabs: Option<Res<CubePrefabsResource>>,
    mut commands: Commands,
    mut spawned: Local<Spawned>,
    handle: Res<CubePrefabSpawnSettingsAsset>,
    settings: ResMut<Assets<CubePrefabSpawnSettings>>,
) {
    tracing::info!("running cube spawner system");
    if spawned.done {
        return;
    }

    let Some(settings) = settings.get(&handle.0) else {
        tracing::info!("missing cube settings resource");
        return;
    };

    let Some(prefabs) = prefabs else {
        tracing::info!("missing cube prefabs");
        return;
    };

    let Some(hello) = prefabs.get_unity_prefab(&crate::types::CubePrefabs::HelloCube) else {
        tracing::info!("missing hello cube prefab");
        return;
    };

    tracing::info!("spawning {} cubes", settings.spawn_count);

    let mut rng = rand::thread_rng();
    let mut i = 0;
    while i < settings.spawn_count {
        i += 1;

        let mut entity = commands.spawn_empty();
        entity.insert(*hello);
        let x: f32 = -20.0 + rng.gen::<f32>() * 40.0;
        let z: f32 = -20.0 + rng.gen::<f32>() * 40.0;

        entity.insert(TransformBundle::from_transform(
            Transform::from_translation(Vec3 { x, y: 10.0, z }),
        ));
    }

    spawned.done = true;
}

pub(crate) fn fall_system(
    mut cubes: Query<(Entity, &mut Transform, &DoRotate, &UnityEntity), Without<Parent>>,
    mut commands: Commands,
    timer: Res<Time>,
) {
    for (entity, mut transform, _, unity_entity) in &mut cubes {
        transform.translation += Vec3 {
            x: 0.0,
            y: -timer.delta_seconds(),
            z: 0.0,
        };

        if transform.translation.y < -5.0 {
            commands.entity(entity).despawn_recursive();
            commands.spawn(unrust::DestroyEntity {
                entity: *unity_entity,
            }); // tells unity to destroy this too
        }
    }
}
