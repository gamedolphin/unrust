using Unity.Entities;
using UnityEngine;
using unrust.runtime;

namespace unrust.userland
{
    public class CubePrefabsAuthoring : MonoBehaviour {
        public GameObject HelloCube;

        public const int RESOURCE_ID = 0;

        class Baker : Baker<CubePrefabsAuthoring>
        {
            public override void Bake(CubePrefabsAuthoring authoring)
            {
                var containerEntity = GetEntity(TransformUsageFlags.None);
                var buffer = AddBuffer<UnrustSpawnable>(containerEntity).Reinterpret<Entity>();

                buffer.Add(GetEntity(authoring.HelloCube, TransformUsageFlags.Dynamic));

                AddComponent<UnrustResourceID>(containerEntity, new UnrustResourceID { Value = 0});
            }
        }
    }
}
