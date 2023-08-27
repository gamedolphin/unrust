use std::{sync::Arc, time::Duration};

use bevy::{
    app::PluginGroupBuilder,
    asset::{AssetPlugin, ChangeWatcher},
    prelude::*,
    tasks::tick_global_task_pools_on_main_thread,
    time::TimePlugin,
};
use inbuilt::*;

pub type UpdateFn = extern "C" fn(data: *const InbuiltEntityData, len: usize);
pub type CreateFn = extern "C" fn(data: *const InbuiltEntityData, len: usize);
pub type DestroyFn = extern "C" fn(entity: *const UnityEntity, len: usize);

#[derive(Component, Copy, Clone)]
pub struct InstantiateEntity {
    pub entity: UnityEntity,
}

#[derive(Component, Copy, Clone)]
pub struct DestroyEntity {
    pub entity: UnityEntity,
}

#[derive(Component)]
pub struct PrefabComponent {
    pub ref_id: i32,
    pub guid: UnityEntity,
}

#[repr(C)]
pub struct PrefabData {
    pub ref_id: i32,
    pub guids: *mut UnityEntity,
    pub len: usize,
}

#[derive(Resource)]
pub struct CallbacksNonSend {
    create_fn: Arc<CreateFn>,
    update_fn: Arc<UpdateFn>,
    destroy_fn: Arc<DestroyFn>,
}

pub(crate) struct UnityPlugins {
    base_path: String,
    update_fn: Arc<UpdateFn>,
    create_fn: Arc<CreateFn>,
    destroy_fn: Arc<DestroyFn>,
}

impl UnityPlugins {
    pub(crate) fn new(
        base_path: String,
        create_fn: CreateFn,
        update_fn: UpdateFn,
        destroy_fn: DestroyFn,
    ) -> UnityPlugins {
        UnityPlugins {
            base_path,
            update_fn: Arc::new(update_fn),
            create_fn: Arc::new(create_fn),
            destroy_fn: Arc::new(destroy_fn),
        }
    }
}

impl Plugin for UnityPlugins {
    fn build(&self, app: &mut App) {
        let update_fn = self.update_fn.clone();
        let create_fn = self.create_fn.clone();
        let destroy_fn = self.destroy_fn.clone();
        app.add_plugins(BasePlugins)
            .add_plugins(AssetPlugin {
                asset_folder: self.base_path.clone(),
                watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
            })
            .insert_non_send_resource(CallbacksNonSend {
                create_fn,
                update_fn,
                destroy_fn,
            })
            .add_systems(
                PostUpdate,
                (
                    create_unity_system,
                    update_unity_system,
                    destroy_unity_system,
                ),
            );
    }
}

fn create_unity_system(
    callbacks_res: NonSend<CallbacksNonSend>,
    mut commands: Commands,
    created: Query<(Entity, &InstantiateEntity, &Transform)>,
) {
    let mut creates = vec![];
    let mut counts = vec![];
    created.iter().for_each(|(_, _, transform)| {
        let mut count = 0;
        creates.push(InbuiltData {
            ty: InbuiltTypes::UnityTransform,
            value: InbuiltComponents {
                UnityTransform: transform.into(),
            },
        });

        count += 1;
        counts.push(count);
    });

    let mut count = 0;
    let creates = created
        .iter()
        .enumerate()
        .map(|(index, (entity, guid, _))| {
            let comp_count = counts[index];
            let ptr = creates[count..(count + comp_count)].as_ptr();
            count += comp_count;

            let mut comm = commands.entity(entity);
            comm.despawn();

            InbuiltEntityData {
                entity: guid.entity,
                data: ptr,
                len: comp_count,
            }
        })
        .collect::<Vec<InbuiltEntityData>>();

    let ptr = creates.as_ptr();

    (callbacks_res.create_fn)(ptr, creates.len());
}

fn update_unity_system(
    callbacks_res: NonSend<CallbacksNonSend>,
    entities: Query<(&UnityEntity, &GlobalTransform), Changed<GlobalTransform>>, // add more inbuilt components here
) {
    let mut updates = vec![];
    let mut counts = vec![];
    entities.iter().for_each(|(_, transform)| {
        let mut count = 0;
        updates.push(InbuiltData {
            ty: InbuiltTypes::UnityTransform,
            value: InbuiltComponents {
                UnityTransform: transform.into(),
            },
        });

        count += 1;
        counts.push(count);
    });

    let mut count = 0;
    let updates = entities
        .iter()
        .enumerate()
        .map(|(index, (unity_entity, _))| {
            let comp_count = counts[index];
            let ptr = updates[count..(count + comp_count)].as_ptr();
            count += comp_count;

            InbuiltEntityData {
                entity: *unity_entity,
                data: ptr,
                len: comp_count,
            }
        })
        .collect::<Vec<InbuiltEntityData>>();

    let ptr = updates.as_ptr();
    (callbacks_res.update_fn)(ptr, updates.len());
}

fn destroy_unity_system(
    mut commands: Commands,
    callbacks_res: NonSend<CallbacksNonSend>,
    destroyed: Query<(Entity, &DestroyEntity)>,
) {
    let destroyed_entities = destroyed
        .iter()
        .map(|(entity, v)| {
            let mut comm = commands.entity(entity);
            comm.despawn();

            v.entity
        })
        .collect::<Vec<UnityEntity>>();

    (callbacks_res.destroy_fn)(destroyed_entities.as_ptr(), destroyed_entities.len());
}

pub(crate) fn start_app(app: &mut App) {
    // copy of schedule
    while !app.ready() {
        #[cfg(not(target_arch = "wasm32"))]
        tick_global_task_pools_on_main_thread();
    }
    app.finish();
    app.cleanup();
}

struct BasePlugins;

impl PluginGroup for BasePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(TaskPoolPlugin::default())
            .add(TypeRegistrationPlugin)
            .add(FrameCountPlugin)
            .add(TransformPlugin)
            .add(HierarchyPlugin)
            .add(TimePlugin)
    }
}
