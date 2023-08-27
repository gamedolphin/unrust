use genco::prelude::*;
use unrust_proc_macro::unity_authoring;

#[unity_authoring]
pub struct UnityEntity {
    pub index: i32,
    pub version: i32,
}

#[allow(non_snake_case)]
pub fn UnityEntity_ingest_component(_entity: &mut bevy::ecs::world::EntityMut, _val: &UnityEntity) {
}

#[allow(non_snake_case)]
pub fn UnityEntity_CSHARP_TOKEN() -> csharp::Tokens {
    quote! {
        [StructLayout(LayoutKind.Sequential)]
        public struct UnityEntity
        {
            public int Index;
            public int Version;

            public static implicit operator Unity.Entities.Entity(UnityEntity val) => new Unity.Entities.Entity
            {
                Index = val.Index,
                Version = val.Version,
            };

            public static implicit operator UnityEntity(Unity.Entities.Entity val) => new UnityEntity
            {
                Index = val.Index,
                Version = val.Version,
            };
        }
    }
}
