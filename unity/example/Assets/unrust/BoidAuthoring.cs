using System.Runtime.InteropServices;
using Unity.Entities;
using UnityEngine;

namespace unrust.userland
{
    [StructLayout(LayoutKind.Sequential)]
    public struct Boid : IComponentData
    {
        public int speed;
    }

    public class BoidAuthoring : MonoBehaviour
    {
        public int speed;

        class Baker : Baker<BoidAuthoring>
        {
            public override void Bake(BoidAuthoring authoring)
            {
                var entity = GetEntity(TransformUsageFlags.Dynamic);
                AddComponent(entity, new Boid
                    {
                        speed = authoring.speed,
                    });
            }
        }
    }
}
