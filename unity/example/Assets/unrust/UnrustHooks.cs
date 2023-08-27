using Unity.Entities;
using UnityEngine;
using unrust.runtime;

namespace unrust.userland
{
    public static class UnrustHooks
    {
        [RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.BeforeSceneLoad)]
        static void Initialize()
        {
            NativeWrapper.CustomCreates = CreateCallback;
        }

        public static unsafe ulong CreateCallback(EntityManager manager, Entity entity, CustomCreateCallback cb)
        {
            var count = 0;
            var arr = new CustomData[CustomComponents.ComponentCount];

            if (manager.HasComponent<SampleComponent>(entity))
            {
                arr[count] = new CustomData
                {
                    ty = CustomType.SampleComponent,
                    value = new CustomComponents { SampleComponent = manager.GetComponentData<SampleComponent>(entity) },
                };
                count++;
            }

            var stateCount = 0;
            var stateArr = new CustomState[CustomState.CustomStateCount];

            fixed (void* ptr = arr)
            {
                fixed (void* state = stateArr)
                {
                    return cb(ptr, (nuint)count, state, (nuint)stateCount);
                }
            }
        }
    }
}
