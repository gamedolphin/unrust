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

            if (manager.HasComponent<DoRotate>(entity))
            {
                arr[count] = new CustomData
                {
                    ty = CustomType.DoRotate,
                    value = new CustomComponents { DoRotate = manager.GetComponentData<DoRotate>(entity) },
                };
                count++;
            }if (manager.HasComponent<Boid>(entity))
            {
                arr[count] = new CustomData
                {
                    ty = CustomType.Boid,
                    value = new CustomComponents { Boid = manager.GetComponentData<Boid>(entity) },
                };
                count++;
            }

            var stateCount = 0;
            var stateArr = new CustomState[CustomState.CustomStateCount];

            if (manager.HasComponent<GameState>(entity))
            {
                stateArr[stateCount] = new CustomState
                {
                    ty = CustomStateType.GameState,
                    value = manager.GetComponentData<GameState>(entity).Value,
                };
                stateCount++;
            }

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
