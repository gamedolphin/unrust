using System.Runtime.InteropServices;
using Unity.Entities;
using UnityEngine;

namespace unrust.userland
{
    [StructLayout(LayoutKind.Sequential)]
    public struct SampleComponent : IComponentData
    {
        public float speed;
    }

    public class SampleComponentAuthoring : MonoBehaviour
    {
        public float speed;

        class Baker : Baker<SampleComponentAuthoring>
        {
            public override void Bake(SampleComponentAuthoring authoring)
            {
                var entity = GetEntity(TransformUsageFlags.Dynamic);
                AddComponent(entity, new SampleComponent
                    {
                        speed = authoring.speed,
                    });
            }
        }
    }
}
