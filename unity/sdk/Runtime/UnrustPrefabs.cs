using UnityEngine;
using Unity.Entities;

namespace unrust.runtime
{
    public struct UnrustSpawnable : IBufferElementData
    {
        public Entity Value;
    }

    public struct UnrustResourceID : IComponentData
    {
        public int Value;
    }

    public struct UnrustPrefabIndex : IComponentData
    {
        public int Value;
    }
}
