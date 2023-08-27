using Unity.Entities;
using UnityEngine;
using unrust.runtime;

namespace unrust.userland
{
    public class BoidPrefabsAuthoring : MonoBehaviour {
        public GameObject Boid;

        public const int RESOURCE_ID = 1;

        class Baker : Baker<BoidPrefabsAuthoring>
        {
            public override void Bake(BoidPrefabsAuthoring authoring)
            {
                var containerEntity = GetEntity(TransformUsageFlags.None);
                var buffer = AddBuffer<UnrustSpawnable>(containerEntity).Reinterpret<Entity>();

                buffer.Add(GetEntity(authoring.Boid, TransformUsageFlags.Dynamic));

                AddComponent<UnrustResourceID>(containerEntity, new UnrustResourceID { Value = 1});
            }
        }
    }
}
