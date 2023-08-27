use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::fmt::Write;

#[proc_macro_attribute]
pub fn unity_authoring(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(item);
    quote! {
        #[derive(bevy::prelude::Component, Clone, Copy, Debug)]
        #[repr(C)]
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn bevy_state(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(item);
    quote! {
        #[repr(u8)]
        #[derive(Clone, Eq, PartialEq, Debug, Hash, Default, unrust::bevy::prelude::States)]
        #input
    }
    .into()
}

#[proc_macro]
pub fn generate_inbuilt(item: TokenStream) -> TokenStream {
    let config = syn::parse2::<syn::ExprTuple>(item.into()).expect("expecting tuples");

    let enum_types = config.elems.iter().filter_map(|p| {
        let syn::Expr::Path(p) = p else {
            return None;
        };
        let p = p.path.get_ident()?;
        Some(p)
    });

    let custom_types = enum_types.clone().enumerate().map(|(index, ident)| {
        let index = index as u8;
        quote! {
            #ident= #index
        }
    });

    let component_types = enum_types.clone().map(|ident| {
        quote! {
            pub #ident: #ident
        }
    });

    let ingest_types = enum_types.clone().map(|ident| {
        let fn_name = format_ident!("{}_ingest_component", ident);

        quote! {
            InbuiltTypes::#ident => #fn_name(entity, &ele.value.#ident)
        }
    });

    let unity_enums =
        enum_types
            .clone()
            .enumerate()
            .fold(String::default(), |mut acc, (index, ident)| {
                write!(acc, "{} = {},\n", ident, index).unwrap();

                acc
            });

    let union_types = enum_types
        .clone()
        .fold(String::default(), |mut acc, ident| {
            write!(
                acc,
                r#"
              [FieldOffset(0)]
              public {} {};
"#,
                ident, ident
            )
            .unwrap();

            acc
        });

    let count = enum_types.clone().count() as i32;

    let csharp_fns = enum_types.clone().map(|ident| {
        let fn_name = format_ident!("{}_CSHARP_TOKEN", ident);

        quote! {
            #fn_name()
        }
    });

    quote! {
        #[repr(u8)]
        pub enum InbuiltTypes {
            #(#custom_types,)*
        }

        #[allow(non_snake_case)]
        pub union InbuiltComponents {
            #(#component_types,)*
        }

        #[repr(C)]
        pub struct InbuiltData {
            pub ty: InbuiltTypes,
            pub value: InbuiltComponents,
        }

        #[repr(C)]
        pub struct InbuiltEntityData {
            pub entity: UnityEntity,
            pub data: *const InbuiltData,
            pub len: usize,
        }

        pub unsafe fn ingest_component(entity: &mut EntityMut, components: &[InbuiltData]) {
            for ele in components {
                match ele.ty {
                    #(#ingest_types,)*
                }
            }
        }

        const UNITY_TYPES: &str = #unity_enums;
        const UNITY_UNION: &str = #union_types;
        const UNITY_COUNT: i32 = #count;

        fn get_inbuilt_csharp_tokens() -> Vec<csharp::Tokens> {
            vec![#(#csharp_fns,)*]
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn unity_prefab(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(item.clone());

    let parsed_enum = syn::parse_macro_input!(item as syn::ItemEnum);

    let enum_name = parsed_enum.ident;

    let res_name = format_ident!("{}Resource", enum_name);

    let variants = parsed_enum.variants.iter().enumerate().map(|(index, v)| {
        let id = &v.ident;
        quote! {
            self.vals.insert(#enum_name::#id, unrust::InstantiateEntity {
                entity: vals[#index].clone()
            });
        }
    });

    quote! {
        #[derive(PartialEq,Eq, Hash)]
        #input

        #[derive(unrust::bevy::prelude::Resource, Default)]
        pub struct #res_name {
            pub vals: std::collections::HashMap<#enum_name,unrust::InstantiateEntity>
        }

        impl #res_name {
            pub fn get_unity_prefab(&self, val: &#enum_name) -> Option<&unrust::InstantiateEntity> {
                self.vals.get(val)
            }

            pub fn insert_prefabs(&mut self, vals: &[unrust::UnityEntity]) {
                #(#variants)*
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn unrust_setup(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(item.clone());
    let parsed = syn::parse_macro_input!(item as syn::ItemFn);
    let ident = parsed.sig.ident;

    let custom_incoming = handle_custom_components(attr.clone());
    let state_incoming = handle_custom_states(attr.clone());

    let states = custom_states(attr.clone());
    let prefabs = prefab_resources(attr.clone());
    let register = register_prefabs(attr.clone());

    quote! {
        #input

        #[derive(Default,Copy,Clone)]
        #[repr(C)]
        pub struct Game;

        impl GamePlugin for Game {
            fn initialize(&self, app: &mut App) {
                #states
                #prefabs
                #ident(app);
            }

            fn register(&self, world: &mut World, prefabs: unrust::PrefabData) {
                #register
            }

            #[allow(clippy::missing_safety_doc)]
            unsafe fn spawn_custom(
                &self,
                entity: &mut unrust::bevy::ecs::world::EntityMut,
                custom: *const u8,
                custom_len: usize,
                custom_state: *const u8,
                custom_state_len: usize,
            ) {
                unsafe { handle_custom_components(entity, custom, custom_len) };
                unsafe { handle_custom_states(entity, custom_state, custom_state_len); }
            }
        }

        #custom_incoming

        #state_incoming

        #[no_mangle]
        pub extern "C" fn create_game() {
            let game = Box::new(Game);

            unsafe { unrust::setup_game(game) };
        }
    }
    .into()
}

fn get_nth_tuple(item: TokenStream, n: usize) -> Option<syn::ExprTuple> {
    let tokens = proc_macro2::TokenStream::from(item);
    let config = syn::parse2::<syn::ExprTuple>(tokens).expect("expecting a tuple of tuples");

    let Some(config) = config.elems.iter().nth(n) else {
        panic!("expected at least n tuples")
    };

    let syn::Expr::Tuple(types) = config else {
        panic!("expected tuple for custom incoming types!");
    };

    Some(types.clone())
}

fn handle_custom_components(item: TokenStream) -> proc_macro2::TokenStream {
    let Some(types) = get_nth_tuple(item, 0) else {
        return quote! {
            fn handle_custom_components(entity: &mut unrust::bevy::ecs::world::EntityMut, custom: *const u8, len: usize) {}
        };
    };

    let filtered = types.elems.iter().filter_map(|expr| {
        let syn::Expr::Path(expr) = expr else {
            return None;
        };

        let last = expr.path.segments.last()?;
        Some((last, &expr.path))
    });

    let custom_types = filtered.clone().enumerate().map(|(index, (exp, _))| {
        let index = index as u8;
        quote! {
            #exp = #index
        }
    });

    let component_types = filtered.clone().map(|(exp, rest)| {
        quote! {
            pub #exp: #rest
        }
    });

    let match_types = filtered.clone().map(|(exp, _)| {
        quote! {
            CustomTypes::#exp => entity.insert(ele.value.#exp)
        }
    });

    if filtered.count() > 0 {
        quote! {
            #[repr(u8)]
            pub enum CustomTypes {
                #(#custom_types,)*
            }

            #[allow(non_snake_case)]
            union CustomComponents {
                #(#component_types,)*
            }

            #[repr(C)]
            struct CustomData {
                pub ty: CustomTypes,
                pub value: CustomComponents,
            }

            unsafe fn handle_custom_components(entity: &mut unrust::bevy::ecs::world::EntityMut, custom: *const u8, len: usize) {
                let components = unsafe { std::slice::from_raw_parts(custom as *const CustomData, len) };
                for ele in components {
                    match ele.ty {
                        #(#match_types,)*
                    };
                };
            }
        }
    } else {
        quote! {
            fn handle_custom_components(entity: &mut unrust::bevy::ecs::world::EntityMut, custom: *const u8, len: usize) {}
        }
    }
}

fn handle_custom_states(item: TokenStream) -> proc_macro2::TokenStream {
    let Some(types) = get_nth_tuple(item, 1) else {
        return quote! {
            fn handle_custom_components(entity: &mut unrust::bevy::ecs::world::EntityMut, custom: *const u8, len: usize) {}
        };
    };

    let filtered = types.elems.iter().filter_map(|expr| {
        let syn::Expr::Path(expr) = expr else {
            return None;
        };

        let last = expr.path.segments.last()?;
        Some((last, &expr.path))
    });

    let custom_types = filtered.clone().enumerate().map(|(index, (exp, _))| {
        let index = index as u8;
        quote! {
            #exp = #index
        }
    });

    let match_types = filtered.clone().map(|(exp, _)| {
        let ident = &exp.ident;
        let comp_name = format_ident!("Custom{ident}");
        quote! {
            CustomStates::#exp => {
                entity.insert(#comp_name { val: ele.value });
            }
        }
    });

    let custom_components = filtered.clone().map(|(exp, p)| {
        let ident = &exp.ident;
        let comp_name = format_ident!("Custom{ident}");
        let fn_name = format_ident!("update_{ident}");
        quote! {
            #[derive(unrust::bevy::prelude::Component,Debug, Clone, PartialEq)]
            pub struct #comp_name {
                pub val: u8,
            }

            #[allow(non_snake_case)]
            fn #fn_name(
                mut next_state: ResMut<NextState<#p>>,
                entities: Query<&#comp_name, Changed<#comp_name>>,
            ) {
                let Ok(state) = entities.get_single() else {
                    return;
                };

                unrust::tracing::trace!("switching to {}", state.val);

                unsafe {
                    next_state.set(std::mem::transmute(state.val));
                }
            }
        }
    });

    if filtered.count() > 0 {
        quote! {
            #(#custom_components)*

            #[repr(u8)]
            #[derive(Clone, Debug, PartialEq)]
            pub enum CustomStates {
                #(#custom_types,)*
            }

            #[repr(C)]
            struct CustomStateData {
                pub ty: CustomStates,
                pub value: u8,
            }

            unsafe fn handle_custom_states(entity: &mut unrust::bevy::ecs::world::EntityMut, custom: *const u8, len: usize) {
                let components = unsafe { std::slice::from_raw_parts(custom as *const CustomStateData, len) };
                for ele in components {
                    match ele.ty {
                        #(#match_types,)*
                    };
                };
            }
        }
    } else {
        quote! {
            unsafe fn handle_custom_states(entity: &mut unrust::bevy::ecs::world::EntityMut, custom: *const u8, len: usize) {}
        }
    }
}

fn custom_states(item: TokenStream) -> proc_macro2::TokenStream {
    let Some(states) = get_nth_tuple(item, 1) else {
        return quote! {};
    };

    let filtered = states.elems.iter().filter_map(|expr| {
        let syn::Expr::Path(expr) = expr else {
            return None;
        };

        let last = expr.path.segments.last()?;
        let comp_name = &last.ident;
        let fn_name = format_ident!("update_{comp_name}");

        Some(quote! {
            app.add_state::<#expr>();
            app.add_systems(PostUpdate, #fn_name);
        })
    });

    quote! {
        #(#filtered)*
    }
}

fn prefab_resources(item: TokenStream) -> proc_macro2::TokenStream {
    let Some(states) = get_nth_tuple(item, 2) else {
        return quote! {};
    };

    let filtered = states.elems.iter().filter_map(|expr| {
        let syn::Expr::Path(expr) = expr else {
            return None;
        };

        let last = expr.path.segments.last()?;
        let comp_name = &last.ident;
        let res_name = format_ident!("{comp_name}Resource");

        Some(quote! {
            app.insert_resource(#res_name::default());
        })
    });

    quote! {
        #(#filtered)*
    }
}

fn register_prefabs(item: TokenStream) -> proc_macro2::TokenStream {
    let Some(states) = get_nth_tuple(item, 2) else {
        return quote! {};
    };

    let filtered = states
        .elems
        .iter()
        .filter_map(|expr| {
            let syn::Expr::Path(expr) = expr else {
                return None;
            };

            Some(expr)
        })
        .enumerate()
        .map(|(index, ident)| {
            let count = index as i32;
            let ident_name = format!("{:?}", ident);
            let ident = quote! { #ident };
            let ident: syn::Expr =
                syn::parse_str(&format!("{}Resource", ident.to_string().replace(' ', ""))).unwrap();

            Some(quote! {
                #count => {
                    tracing::info!("loading resource prefabs: {} : {}", #count, #ident_name);
                    let Some(mut res) = world.get_resource_mut::<#ident>() else {
                        tracing::warn!("missing expected resource {}",#ident_name);
                        return;
                    };

                    res.insert_prefabs(guids);
                }
            })
        });

    quote! {
        let guids = unsafe { std::slice::from_raw_parts(prefabs.guids, prefabs.len) };
        match prefabs.ref_id {
            #(#filtered)*
            _ => {}
        }
    }
}
