use std::ffi::{c_char, CStr, CString};

mod loader;
mod logger;
mod unity;

pub use bevy;

use bevy::prelude::*;
pub use codegen::generate_csharp;
use inbuilt::{ingest_component, InbuiltData, UnityEntity};
pub use loader::GamePlugin;

pub use unity::{CreateFn, DestroyEntity, DestroyFn, InstantiateEntity, PrefabData, UpdateFn};
pub use unrust_proc_macro::*;

use crate::{
    logger::{setup_logging, teardown_logging, LoggerFunc},
    unity::{start_app, UnityPlugins},
};

#[repr(C)]
pub struct UnrustContextWrapper;

pub struct UnrustContext {
    pub app: App,
}

static mut GAMEPLUGIN: Option<Box<dyn GamePlugin>> = None;

#[allow(clippy::missing_safety_doc)]
pub unsafe fn setup_game(game: Box<dyn GamePlugin>) {
    GAMEPLUGIN = Some(game);
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn load(logger: LoggerFunc) -> *mut UnrustContextWrapper {
    setup_logging(Box::new(move |level, str| {
        let len = str.len();
        let out_str = CString::new(str).unwrap();
        let out_str = out_str.into_raw();
        (logger)(level, out_str, len);
        unsafe {
            let _ = CString::from_raw(out_str); // drop the string
        }
    }));

    let app = App::new();

    tracing::info!("setting up!");

    let ctx = Box::new(UnrustContext { app });

    Box::into_raw(ctx) as *mut UnrustContextWrapper
}

#[no_mangle]
pub extern "C" fn init(
    ctx: *mut UnrustContextWrapper,
    base_path: *const c_char,
    create: CreateFn,
    update: UpdateFn,
    destroy: DestroyFn,
) {
    tracing::info!("called init");
    let ctx = unsafe { Box::leak(Box::from_raw(ctx as *mut UnrustContext)) };
    let base_path = unsafe { get_string(base_path) };
    ctx.app
        .add_plugins(UnityPlugins::new(base_path, create, update, destroy));

    unsafe {
        if let Some(game) = &GAMEPLUGIN {
            tracing::info!("calling initialize on game!");
            game.initialize(&mut ctx.app);
        } else {
            tracing::info!("game  not setup!");
        }
    }

    start_app(&mut ctx.app);
}

#[no_mangle]
pub extern "C" fn register_prefabs(ctx: *mut UnrustContextWrapper, prefabs: PrefabData) {
    let ctx = unsafe { Box::leak(Box::from_raw(ctx as *mut UnrustContext)) };
    unsafe {
        tracing::info!("handing of prefabs to game");
        if let Some(game) = &GAMEPLUGIN {
            game.register(&mut ctx.app.world, prefabs);
        }
    }
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn spawn(
    ctx: *mut UnrustContextWrapper,
    unity_entity: UnityEntity,
    inbuilt: *const InbuiltData,
    len: usize,
    custom: *const u8,
    custom_len: usize,
    custom_state: *const u8,
    custom_state_len: usize,
) -> u64 {
    let components = unsafe { std::slice::from_raw_parts(inbuilt, len) };
    let ctx = unsafe { Box::leak(Box::from_raw(ctx as *mut UnrustContext)) };
    let mut entity = ctx.app.world.spawn_empty();
    entity.insert(unity_entity);
    ingest_component(&mut entity, components);

    unsafe {
        if let Some(game) = &GAMEPLUGIN {
            game.spawn_custom(
                &mut entity,
                custom,
                custom_len,
                custom_state,
                custom_state_len,
            );
        }
    }

    entity.id().to_bits()
}

#[no_mangle]
pub extern "C" fn tick(ctx: *mut UnrustContextWrapper) {
    let ctx = unsafe { Box::leak(Box::from_raw(ctx as *mut UnrustContext)) };
    ctx.app.update();
}

#[no_mangle]
pub extern "C" fn unload(ctx: *mut UnrustContextWrapper) {
    teardown_logging();
    let _ = unsafe { Box::from_raw(ctx as *mut UnrustContext) };
    unsafe { GAMEPLUGIN = None };
}

unsafe fn get_string(base_path: *const i8) -> String {
    CStr::from_ptr(base_path).to_string_lossy().to_string()
}
