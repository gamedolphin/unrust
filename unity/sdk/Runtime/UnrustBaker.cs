using UnityEngine;
using Unity.Entities;

namespace unrust.runtime
{
    public partial class UnrustAuthoring : MonoBehaviour
    {
        partial class Baker : Baker<UnrustAuthoring>
        {
            public override void Bake(UnrustAuthoring authoring)
            {
                var entity = GetEntity(TransformUsageFlags.Dynamic);
                AddComponent(entity, new CreateUnrust { });
            }
        }
    }
}
