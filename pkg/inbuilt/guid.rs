use genco::prelude::*;
use unrust_proc_macro::unity_authoring;

#[unity_authoring]
pub struct UnityGUID {
    pub hash: [u32; 4],
}

#[allow(non_snake_case)]
pub fn UnityGUID_ingest_component(entity: &mut bevy::ecs::world::EntityMut, val: &UnityGUID) {
    entity.insert(*val);
}

#[allow(non_snake_case)]
pub fn UnityGUID_CSHARP_TOKEN() -> csharp::Tokens {
    quote! {
        [StructLayout(LayoutKind.Sequential)]
        public unsafe struct UnityGUID
        {
            public fixed uint hash[4];

            public static implicit operator Unity.Entities.Serialization.EntityPrefabReference(UnityGUID val) => new Unity.Entities.Serialization.EntityPrefabReference(new Unity.Entities.Hash128(val.hash[0], val.hash[1], val.hash[2], val.hash[3]));

            public static implicit operator UnityGUID(Unity.Entities.Serialization.EntityPrefabReference val)
            {
                var guid = new UnityGUID();
                var hashes = val.AssetGUID.Value;
                guid.hash[0] = hashes[0];
                guid.hash[1] = hashes[1];
                guid.hash[2] = hashes[2];
                guid.hash[3] = hashes[3];
                return guid;
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        public unsafe struct UnityPrefab
        {
            public int ResourceID;
            public UnityEntity* Guid;
            public nuint Length;
        }
    }
}
