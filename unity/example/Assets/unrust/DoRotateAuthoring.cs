using System.Runtime.InteropServices;
using Unity.Entities;
using UnityEngine;

namespace unrust.userland
{
    [StructLayout(LayoutKind.Sequential)]
    public struct DoRotate : IComponentData
    {
        public float speed;
    }

    public class DoRotateAuthoring : MonoBehaviour
    {
        public float speed;

        class Baker : Baker<DoRotateAuthoring>
        {
            public override void Bake(DoRotateAuthoring authoring)
            {
                var entity = GetEntity(TransformUsageFlags.Dynamic);
                AddComponent(entity, new DoRotate
                    {
                        speed = authoring.speed,
                    });
            }
        }
    }
}
