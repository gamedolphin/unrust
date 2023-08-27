using System.Runtime.InteropServices;

namespace unrust.runtime
{
    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct UnityParent
    {
        public ulong parent;
    }[StructLayout(LayoutKind.Sequential)]
    public struct UnityEntity
    {
        public int Index;
        public int Version;

        public static implicit operator Unity.Entities.Entity(UnityEntity val) => new Unity.Entities.Entity
        {
            Index = val.Index,
            Version = val.Version,
        };

        public static implicit operator UnityEntity(Unity.Entities.Entity val) => new UnityEntity
        {
            Index = val.Index,
            Version = val.Version,
        };
    }[StructLayout(LayoutKind.Sequential)]
    public unsafe struct UnityGUID
    {
        public fixed uint hash[4];

        public static implicit operator Unity.Entities.Serialization.EntityPrefabReference(UnityGUID val) => new Unity.Entities.Serialization.EntityPrefabReference(new Unity.Entities.Hash128(val.hash[0], val.hash[1], val.hash[2], val.hash[3]));

        public static implicit operator UnityGUID(Unity.Entities.Serialization.EntityPrefabReference val)
        {
            var guid = new UnityGUID();
            var hashes = val.AssetGUID.Value;
            guid.hash[0] = hashes[0];
            guid.hash[1] = hashes[1];
            guid.hash[2] = hashes[2];
            guid.hash[3] = hashes[3];
            return guid;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct UnityPrefab
    {
        public int ResourceID;
        public UnityEntity* Guid;
        public nuint Length;
    }[StructLayout(LayoutKind.Sequential)]
    public unsafe struct UnityTransform
    {
        public fixed float matrix[16];

        public static implicit operator Unity.Transforms.LocalTransform(UnityTransform val) => Unity.Transforms.LocalTransform.FromMatrix(new Unity.Mathematics.float4x4(
            val.matrix[0], val.matrix[4], val.matrix[8], val.matrix[12],
            val.matrix[1], val.matrix[5], val.matrix[9], val.matrix[13],
            val.matrix[2], val.matrix[6], val.matrix[10], val.matrix[14],
            val.matrix[3], val.matrix[7], val.matrix[11], val.matrix[15]
        ));

        public static implicit operator Unity.Transforms.LocalToWorld(UnityTransform val) => new Unity.Transforms.LocalToWorld
        {
            Value = new Unity.Mathematics.float4x4(
                val.matrix[0], val.matrix[4], val.matrix[8], val.matrix[12],
                val.matrix[1], val.matrix[5], val.matrix[9], val.matrix[13],
                val.matrix[2], val.matrix[6], val.matrix[10], val.matrix[14],
                val.matrix[3], val.matrix[7], val.matrix[11], val.matrix[15]
            )
        };

        public static implicit operator UnityTransform(Unity.Transforms.LocalTransform output)
        {
            var transform = new UnityTransform();
            var val = output.ToMatrix();
            transform.matrix[0] = val.c0[0];
            transform.matrix[1] = val.c0[1];
            transform.matrix[2] = val.c0[2];
            transform.matrix[3] = val.c0[3];
            transform.matrix[4] = val.c1[0];
            transform.matrix[5] = val.c1[1];
            transform.matrix[6] = val.c1[2];
            transform.matrix[7] = val.c1[3];
            transform.matrix[8] = val.c2[0];
            transform.matrix[9] = val.c2[1];
            transform.matrix[10] = val.c2[2];
            transform.matrix[11] = val.c2[3];
            transform.matrix[12] = val.c3[0];
            transform.matrix[13] = val.c3[1];
            transform.matrix[14] = val.c3[2];
            transform.matrix[15] = val.c3[3];

            return transform;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct UnityData
    {
        public UnityTypes ty;
        public UnityComponents value;
    }

    public enum UnityTypes : sbyte
    {
        UnityParent = 0,
UnityEntity = 1,
UnityGUID = 2,
UnityTransform = 3,

    }

    [StructLayout(LayoutKind.Explicit)]
    public unsafe struct UnityComponents
    {
        public const int ComponentCount = 4;

        
              [FieldOffset(0)]
              public UnityParent UnityParent;

              [FieldOffset(0)]
              public UnityEntity UnityEntity;

              [FieldOffset(0)]
              public UnityGUID UnityGUID;

              [FieldOffset(0)]
              public UnityTransform UnityTransform;

    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct EntityData
    {
        public UnityEntity entity;
        public UnityData* data;
        public nuint len;
    }
}
