use anyhow::Context;
use anyhow::Result;
use genco::fmt;
use genco::prelude::*;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

pub fn generate_csharp(path: &str, base_folder: &str) -> Result<()> {
    clear_contents(base_folder, "cs")?;

    let contents = std::fs::read_to_string(path)?;
    let ast = syn::parse_file(&contents)?;

    let custom_comps = generate_components_csharp(ast.clone(), base_folder)?;
    let custom_states = generate_states_csharp(ast.clone(), base_folder)?;
    generate_prefabs_csharp(ast.clone(), base_folder)?;

    generate_hooks(base_folder, custom_comps, custom_states)?;

    Ok(())
}

fn generate_hooks(
    base_folder: &str,
    custom_comps: csharp::Tokens,
    custom_states: csharp::Tokens,
) -> Result<()> {
    let runtime_initialize = &csharp::import("UnityEngine", "RuntimeInitializeOnLoadMethod");
    let runtime_initialize_load = &csharp::import("UnityEngine", "RuntimeInitializeLoadType");
    let unrust_native = &csharp::import("unrust.runtime", "NativeWrapper");
    let entity_manager = &csharp::import("Unity.Entities", "EntityManager");
    let entity = &csharp::import("Unity.Entities", "Entity");
    let create_callback = &csharp::import("unrust.runtime", "CustomCreateCallback");

    let hooks: csharp::Tokens = quote! {
        namespace unrust.userland
        {
            public static class UnrustHooks
            {
                [$runtime_initialize($runtime_initialize_load.BeforeSceneLoad)]
                static void Initialize()
                {
                    $unrust_native.CustomCreates = CreateCallback;
                }

                public static unsafe ulong CreateCallback($entity_manager manager, $entity entity, $create_callback cb)
                {
                    var count = 0;
                    var arr = new CustomData[CustomComponents.ComponentCount];

                    $(custom_comps)

                    var stateCount = 0;
                    var stateArr = new CustomState[CustomState.CustomStateCount];

                    $(custom_states)

                    fixed (void* ptr = arr)
                    {
                        fixed (void* state = stateArr)
                        {
                            return cb(ptr, (nuint)count, state, (nuint)stateCount);
                        }
                    }
                }
            }
        }
    };

    write_tokens_to_file(base_folder, "UnrustHooks.cs", hooks)
}

fn generate_prefabs_csharp(ast: syn::File, base_folder: &str) -> Result<()> {
    let monobehaviour = &csharp::import("UnityEngine", "MonoBehaviour");
    let baker = &csharp::import("Unity.Entities", "Baker");
    let spawnable = &csharp::import("unrust.runtime", "UnrustSpawnable");

    let enums = ast
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Enum(s) => Some(s),
            _ => None,
        })
        .filter(|item| {
            item.attrs
                .iter()
                .any(|attr| attr.path().is_ident("unity_prefab"))
        })
        .map(|item| {
            (
                item.ident.to_string(),
                item.variants.iter().map(|v| v.ident.to_string()),
            )
        })
        .enumerate()
        .map(|(index, (enum_name, variants))| {
            let variant_fields = variants.clone().map(|name| {
                quote! {
                    $['\r']public GameObject $(&name);
                }
            });

            let count = index;

            let author_fields = variants.map(|name| {
                quote! {
                    buffer.Add(GetEntity(authoring.$(&name), TransformUsageFlags.Dynamic));
                }
            });

            let comp: csharp::Tokens = quote! {
                namespace unrust.userland
                {
                    public class $(&enum_name)Authoring : $monobehaviour {
                        $(for n in variant_fields => $n)

                        public const int RESOURCE_ID = $count;

                        class Baker : $baker<$(&enum_name)Authoring>
                        {
                            public override void Bake($(&enum_name)Authoring authoring)
                            {
                                var containerEntity = GetEntity(TransformUsageFlags.None);
                                var buffer = AddBuffer<$spawnable>(containerEntity).Reinterpret<Entity>();

                                $(for n in author_fields => $n)

                                AddComponent<UnrustResourceID>(containerEntity, new UnrustResourceID { Value =  $count});
                            }
                        }
                    }
                }
            };

            (enum_name, comp)
        });

    enums.clone().try_for_each(|(name, comp)| {
        write_tokens_to_file(base_folder, &format!("{}Authoring.cs", name), comp)
    })?;

    Ok(())
}

fn generate_states_csharp(ast: syn::File, base_folder: &str) -> Result<csharp::Tokens> {
    let struct_layout = &csharp::import("System.Runtime.InteropServices", "StructLayout");
    let layout_kind = &csharp::import("System.Runtime.InteropServices", "LayoutKind");
    let monobehaviour = &csharp::import("UnityEngine", "MonoBehaviour");
    let component_data = &csharp::import("Unity.Entities", "IComponentData");

    let enums = ast
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Enum(s) => Some(s),
            _ => None,
        })
        .filter(|item| {
            item.attrs
                .iter()
                .any(|attr| attr.path().is_ident("bevy_state"))
        })
        .map(|item| (item.ident.to_string(), &item.variants))
        .map(|(enum_name, enum_variants)| {
            let enum_fields = enum_variants.iter();

            let name_list = enum_fields.clone().enumerate().map(|(index, v)| {
                quote! {
                    $['\r']$(v.ident.to_string()) = $index,
                }
            });

            let comp: csharp::Tokens = quote! {
                namespace unrust.userland
                {
                    [$(struct_layout)($layout_kind.Sequential)]
                    public struct $(&enum_name) : $component_data
                    {
                        public sbyte Value;
                    }

                    public class $(&enum_name)Authoring : $monobehaviour
                    {
                        public enum ENUM_$(&enum_name) : sbyte
                        {
                            $(for n in name_list => $n)
                        }

                        [SerializeField]
                        public ENUM_$(&enum_name) $(&enum_name);

                        class Baker : Baker<$(&enum_name)Authoring>
                        {
                            public override void Bake($(&enum_name)Authoring authoring)
                            {
                                var entity = GetEntity(TransformUsageFlags.None);
                                AddComponent(entity, new $(&enum_name)
                                {
                                        Value = (sbyte)authoring.$(&enum_name)
                                });
                            }
                        }
                    }
                }
            };

            (comp, enum_name)
        });

    enums.clone().try_for_each(|(comp, enum_name)| {
        write_tokens_to_file(base_folder, &format!("{}Authoring.cs", enum_name), comp)
    })?;

    let enum_types = enums.clone().enumerate().map(|(index, (_, name))| {
        quote! {
            $(&name) = $index
        }
    });

    let count = enums.clone().count();

    let generated_comps: csharp::Tokens = quote! {
        namespace unrust.userland
        {
            [$struct_layout($layout_kind.Sequential)]
            public struct CustomState
            {
                public const int CustomStateCount = $count;
                public CustomStateType ty;
                public sbyte value;
            }

            public enum CustomStateType: byte
            {
                $(for n in enum_types => $n)
            }
        }
    };

    write_tokens_to_file(base_folder, "UnrustState.cs", generated_comps)?;

    let add_enums = enums.clone().map(|(_, name)| {
        quote! {
            if (manager.HasComponent<$(&name)>(entity))
            {
                stateArr[stateCount] = new CustomState
                {
                    ty = CustomStateType.$(&name),
                    value = manager.GetComponentData<$(&name)>(entity).Value,
                };
                stateCount++;
            }


        }
    });

    Ok(quote! {
        $(for n in add_enums => $n)
    })
}

fn write_tokens_to_file(base: &str, name: &str, tokens: csharp::Tokens) -> Result<()> {
    let fmt = fmt::Config::from_lang::<Csharp>().with_indentation(fmt::Indentation::Space(4));
    let config = csharp::Config::default();

    let path = Path::new(base);

    let file = File::create(path.join(name)).context("failed to open file")?;
    let mut w = fmt::IoWriter::new(file);
    tokens
        .format_file(&mut w.as_formatter(&fmt), &config)
        .context("could not write to file")
}

fn generate_components_csharp(ast: syn::File, base_folder: &str) -> Result<csharp::Tokens> {
    let structs = find_structs_with_attr(ast, "unity_authoring")
        .into_iter()
        .map(generate_components_with_authoring);

    let fmt = fmt::Config::from_lang::<Csharp>().with_indentation(fmt::Indentation::Space(4));
    let config = csharp::Config::default();

    let path = Path::new(base_folder);

    structs.clone().try_for_each(|(comp, name)| {
        write_tokens_to_file(base_folder, &format!("{}Authoring.cs", name), comp)
    })?;

    let count = structs.clone().count();

    let gen_comps = structs.clone().map(|(_, name)| {
        quote! {
            $['\r'][FieldOffset(0)] public $(&name) $(&name);

        }
    });

    let enum_types = structs.clone().enumerate().map(|(index, (_, name))| {
        quote! {
            $['\r']$name = $index,
        }
    });

    let struct_layout = &csharp::import("System.Runtime.InteropServices", "StructLayout");
    let layout_kind = &csharp::import("System.Runtime.InteropServices", "LayoutKind");

    let generated_comps: csharp::Tokens = quote! {
        namespace unrust.userland
        {
            [$struct_layout($layout_kind.Sequential)]
            public struct CustomData
            {
                public CustomType ty;
                public CustomComponents value;
            }

            public enum CustomType : byte
            {
                $(for n in enum_types  => $n)
            }

            [$struct_layout($layout_kind.Explicit)]
            public struct CustomComponents
            {
                public const int ComponentCount = $count;
                $(for n in gen_comps => $n)
            }
        }
    };

    let file = File::create(path.join("UnrustComponent.cs")).context("failed to open file")?;
    let mut w = fmt::IoWriter::new(file);
    generated_comps
        .format_file(&mut w.as_formatter(&fmt), &config)
        .context("could not write to file")?;

    let add_comps = structs.clone().map(|(_, name)| {
        quote! {


            if (manager.HasComponent<$(&name)>(entity))
            {
                arr[count] = new CustomData
                {
                    ty = CustomType.$(&name),
                    value = new CustomComponents { $(&name) = manager.GetComponentData<$(&name)>(entity) },
                };
                count++;
            }
        }
    });

    let add_comps: csharp::Tokens = quote! {
        $(for n in add_comps  => $n)
    };

    Ok(add_comps)
}

fn generate_components_with_authoring(
    (struct_name, fields): (String, Vec<(String, syn::Type)>),
) -> (csharp::Tokens, String) {
    let component_data = &csharp::import("Unity.Entities", "IComponentData");
    let monobehaviour = &csharp::import("UnityEngine", "MonoBehaviour");
    let struct_layout = &csharp::import("System.Runtime.InteropServices", "StructLayout");
    let layout_kind = &csharp::import("System.Runtime.InteropServices", "LayoutKind");

    let component_fields = fields.iter().map(|(field_name, field_type)| {
        let field_type = map_rust_type(field_type);

        quote! {
            $['\r']public $field_type $field_name;
        }
    });

    let authoring_name = format!("{struct_name}Authoring");

    let authoring_fields = fields.iter().map(|(field_name, _)| {
        quote! {
            $['\r']$field_name = authoring.$field_name,
        }
    });

    let tokens: csharp::Tokens = quote! {
        namespace unrust.userland
        {
            [$struct_layout($layout_kind.Sequential)]
            public struct $(&struct_name) : $component_data
            {
                $(for n in component_fields.clone() => $n)
            }

            public class $(&authoring_name) : $monobehaviour
            {
                $(for n in component_fields => $n)

                class Baker : Baker<$(&authoring_name)>
                {
                    public override void Bake($(&authoring_name) authoring)
                    {
                        var entity = GetEntity(TransformUsageFlags.Dynamic);
                        AddComponent(entity, new $(&struct_name)
                            {
                                $(for n in authoring_fields => $n)
                            });
                    }
                }
            }
        }
    };

    (tokens, struct_name)
}

fn find_structs_with_attr(
    ast: syn::File,
    expected: &str,
) -> Vec<(String, Vec<(String, syn::Type)>)> {
    ast.items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Struct(s) => Some(s),
            _ => None,
        })
        .filter(|item| item.attrs.iter().any(|attr| attr.path().is_ident(expected)))
        .map(|item| {
            let fields = match &item.fields {
                syn::Fields::Named(fields) => fields
                    .named
                    .iter()
                    .map(|f| {
                        let field_name = f.ident.clone().unwrap().to_string();
                        let field_type = f.ty.clone();
                        (field_name, field_type)
                    })
                    .collect::<Vec<(String, syn::Type)>>(),
                _ => vec![],
            };

            (item.ident.to_string(), fields)
        })
        .collect()
}

fn clear_contents(path: &str, extension: &str) -> Result<()> {
    let files = std::fs::read_dir(path)?;
    files
        .filter_map(|p| p.ok())
        .filter_map(|p| {
            let path = p.path();
            let Some(ext) = path.extension() else {
                return None;
            };

            if ext == OsStr::new(extension) {
                Some(p.path())
            } else {
                None
            }
        })
        .try_for_each(std::fs::remove_file)?;

    Ok(())
}

fn map_rust_type(ty: &syn::Type) -> String {
    let syn::Type::Path(path) = ty else {
        panic!("expected rust path type");
    };

    let path = path
        .path
        .get_ident()
        .expect("expected simple path type")
        .to_string();

    match path.as_str() {
        "f32" => "float",
        "f64" => "double",
        "i32" => "int",
        "i64" => "long",
        "u32" => "uint",
        "u64" => "ulong",
        _ => panic!("unsupported base type"),
    }
    .to_string()
}
