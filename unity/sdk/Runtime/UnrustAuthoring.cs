using UnityEngine;
using Unity.Entities;

namespace unrust.runtime
{
    public partial class UnrustAuthoring : MonoBehaviour
    {

    }

    public struct CreateUnrust : IComponentData { }

    [WriteGroup(typeof(Unity.Transforms.LocalToWorld))]
    public struct UnrustEntity : IComponentData
    {
        public ulong ID;
    }
}
