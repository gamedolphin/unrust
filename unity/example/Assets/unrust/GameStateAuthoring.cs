using System.Runtime.InteropServices;
using Unity.Entities;
using UnityEngine;

namespace unrust.userland
{
    [StructLayout(LayoutKind.Sequential)]
    public struct GameState : IComponentData
    {
        public sbyte Value;
    }

    public class GameStateAuthoring : MonoBehaviour
    {
        public enum ENUM_GameState : sbyte
        {
            HelloCubeSimple = 0,
            PrefabCube = 1,
            Parenting = 2,
            Enableable = 3,
            Boids = 4,
        }

        [SerializeField]
        public ENUM_GameState GameState;

        class Baker : Baker<GameStateAuthoring>
        {
            public override void Bake(GameStateAuthoring authoring)
            {
                var entity = GetEntity(TransformUsageFlags.None);
                AddComponent(entity, new GameState
                {
                    Value = (sbyte)authoring.GameState
                });
            }
        }
    }
}
