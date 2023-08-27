mod entity;
mod guid;
mod parent;
mod transform;
use genco::fmt;
use genco::prelude::*;
use std::fs::File;

use bevy::ecs::world::EntityMut;
pub use entity::*;
pub use guid::*;
pub use parent::*;
pub use transform::*;
use unrust_proc_macro::generate_inbuilt;

generate_inbuilt!((UnityParent, UnityEntity, UnityGUID, UnityTransform));

pub fn write_csharp_inbuilt(path: &str) -> anyhow::Result<()> {
    let struct_layout = &csharp::import("System.Runtime.InteropServices", "StructLayout");
    let layout_kind = &csharp::import("System.Runtime.InteropServices", "LayoutKind");
    let unity_types = UNITY_TYPES;
    let unity_union = UNITY_UNION;
    let unity_count = UNITY_COUNT;
    let inbuilt_tokens = get_inbuilt_csharp_tokens();
    let output: csharp::Tokens = genco::prelude::quote! {
        namespace unrust.runtime
        {
            $(for n in inbuilt_tokens => $n)

            [$struct_layout($layout_kind.Sequential)]
            public struct UnityData
            {
                public UnityTypes ty;
                public UnityComponents value;
            }

            public enum UnityTypes : sbyte
            {
                $(unity_types)
            }

            [$struct_layout($layout_kind.Explicit)]
            public unsafe struct UnityComponents
            {
                public const int ComponentCount = $unity_count;

                $(unity_union)
            }

            [$struct_layout($layout_kind.Sequential)]
            public unsafe struct EntityData
            {
                public UnityEntity entity;
                public UnityData* data;
                public nuint len;
            }
        }
    };

    let fmt = fmt::Config::from_lang::<Csharp>().with_indentation(fmt::Indentation::Space(4));
    let config = csharp::Config::default();

    let file = File::create(path)?;
    let mut w = fmt::IoWriter::new(file);
    output.format_file(&mut w.as_formatter(&fmt), &config)?;

    Ok(())
}
