use bevy::prelude::*;
use genco::prelude::*;
use unrust_proc_macro::unity_authoring;

#[unity_authoring]
pub struct UnityParent {
    pub entity: u64,
}

impl From<Entity> for UnityParent {
    fn from(value: Entity) -> Self {
        UnityParent {
            entity: value.to_bits(),
        }
    }
}

impl From<&UnityParent> for Entity {
    fn from(value: &UnityParent) -> Self {
        Self::from_bits(value.entity)
    }
}

impl From<UnityParent> for Entity {
    fn from(value: UnityParent) -> Self {
        Self::from_bits(value.entity)
    }
}

#[allow(non_snake_case)]
pub fn UnityParent_ingest_component(entity: &mut bevy::ecs::world::EntityMut, val: &UnityParent) {
    entity.set_parent(val.into());
    entity.insert(*val);
}

#[allow(non_snake_case)]
pub fn UnityParent_CSHARP_TOKEN() -> csharp::Tokens {
    quote! {
        [StructLayout(LayoutKind.Sequential)]
        public unsafe struct UnityParent
        {
            public ulong parent;
        }
    }
}
