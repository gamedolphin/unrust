using Unity.Entities;
using Unity.Collections;
using System;
using System.Collections.Generic;

namespace unrust.runtime
{
    public unsafe partial class UnrustSystem : SystemBase
    {
        private EntityCommandBuffer ecb;
        private Dictionary<UnrustResourceID, List<UnityEntity>> loadedResources = new Dictionary<UnrustResourceID, List<UnityEntity>>();

        private NativeWrapper nativeWrapper;
        private CreateFunction create;
        private UpdateFunction update;
        private DestroyFunction destroy;


        protected override void OnCreate()
        {
            this.nativeWrapper = new NativeWrapper();

            this.create = OnCreatePrefabs;
            this.update = HandeIncomingUpdates;
            this.destroy = OnDestroyEntity;
            this.nativeWrapper.Initialize(this.create, this.update, this.destroy);
        }

        private void OnDestroyEntity(UnityEntity* data, nuint len)
        {
            var comps = new ReadOnlySpan<UnityEntity>(data, (int)len);
            foreach (var entity in comps)
            {
                ecb.DestroyEntity(entity);
            }
        }

        private void OnCreatePrefabs(EntityData* ptr, nuint length)
        {
            var span = new ReadOnlySpan<EntityData>(ptr, (int)length);
            foreach (var data in span)
            {
                var entity = data.entity;
                var created = ecb.Instantiate(entity);
                var comps = new ReadOnlySpan<UnityData>(data.data, (int)data.len);
                foreach (var comp in comps)
                {
                    switch (comp.ty)
                    {
                        case UnityTypes.UnityTransform:
                            ecb.SetComponent<Unity.Transforms.LocalTransform>(created, comp.value.UnityTransform);
                            break;
                    }
                }
            }
        }

        protected override void OnStartRunning()
        {
            base.OnStartRunning();
            foreach (var (buffer, entity) in SystemAPI.Query<DynamicBuffer<UnrustSpawnable>>().WithEntityAccess())
            {
                var resource_id = SystemAPI.GetComponent<UnrustResourceID>(entity);
                var refs = buffer.Reinterpret<Entity>();
                var guids = new UnityEntity[refs.Length];
                for (int i = 0; i < refs.Length; ++i)
                {
                    guids[i] = refs[i];
                }

                fixed (UnityEntity* ptr = guids)
                {
                    this.nativeWrapper.RegisterPrefabs(new UnityPrefab
                    {
                        ResourceID = resource_id.Value,
                        Guid = ptr,
                        Length = (nuint)refs.Length,
                    });
                }
            }
        }

        protected override void OnDestroy()
        {
            this.nativeWrapper.Dispose();
        }

        private void HandeIncomingUpdates(EntityData* ptr, nuint length)
        {
            var span = new ReadOnlySpan<EntityData>(ptr, (int)length);
            foreach (var data in span)
            {
                var entity = data.entity;
                var comps = new ReadOnlySpan<UnityData>(data.data, (int)data.len);
                foreach (var comp in comps)
                {
                    switch (comp.ty)
                    {
                        case UnityTypes.UnityTransform:
                            ecb.SetComponent<Unity.Transforms.LocalToWorld>(entity, comp.value.UnityTransform);
                            break;
                    }
                }
            }
        }

        protected override void OnUpdate()
        {
            ecb = new EntityCommandBuffer(Allocator.Temp);
            this.HandleNewEntities();
            this.nativeWrapper.Update();
            ecb.Playback(EntityManager);
            ecb.Dispose();
        }

        private Dictionary<Entity, UnrustEntity> idMap = new Dictionary<Entity, UnrustEntity>();
        private List<Entity> heldback = new List<Entity>();
        private void HandleNewEntities()
        {
            var arr = new UnityData[UnityComponents.ComponentCount];
            idMap.Clear();
            heldback.Clear();
            foreach (var (_, entity) in SystemAPI.Query<CreateUnrust>().WithNone<UnrustEntity>().WithEntityAccess())
            {
                if (CreateEntity(arr, entity, idMap, out var bevyEntity))
                {
                    idMap.Add(entity, bevyEntity);
                }
                else
                {
                    heldback.Add(entity);
                }
            }

            var removedIndexes = new List<int>();
            while (heldback.Count > 0) // ensure all heldbacks are also created, for eg: parent references
            {
                removedIndexes.Clear();
                for (int i = heldback.Count - 1; i >= 0; i--)
                {
                    if (CreateEntity(arr, heldback[i], idMap, out var bevyEntity))
                    {
                        idMap.Add(heldback[i], bevyEntity);
                        removedIndexes.Add(i);
                    }
                }

                foreach (var index in removedIndexes)
                {
                    heldback.RemoveAt(index);
                }
            }

            foreach (var (k, v) in idMap)
            {
                ecb.AddComponent(k, v);
            }
        }

        private bool CreateEntity(UnityData[] arr, Entity entity, Dictionary<Entity, UnrustEntity> idMap, out UnrustEntity bevyEntity)
        {
            var (count, ready) = AddComponents(arr, entity, idMap);
            if (!ready) // not ready yet
            {
                bevyEntity = new UnrustEntity { };
                return false;
            }

            fixed (UnityData* comps = arr)
            {
                var bevyId = this.nativeWrapper.Create(this.EntityManager, entity, comps, (nuint)count);
                bevyEntity = new UnrustEntity { ID = bevyId };
                return true;
            }
        }

        private (int, bool) AddComponents(UnityData[] arr, Entity entity, Dictionary<Entity, UnrustEntity> idMap)
        {
            var count = 0;
            if (SystemAPI.HasComponent<Unity.Transforms.Parent>(entity))
            {
                var parent = SystemAPI.GetComponent<Unity.Transforms.Parent>(entity);
                if (!SystemAPI.HasComponent<UnrustEntity>(parent.Value) && !idMap.ContainsKey(parent.Value))
                {
                    return (count, false);
                }


                UnrustEntity parentId;
                if (SystemAPI.HasComponent<UnrustEntity>(parent.Value))
                {
                    parentId = SystemAPI.GetComponent<UnrustEntity>(parent.Value);
                }
                else
                {
                    parentId = idMap[parent.Value];
                }

                arr[count] = new UnityData
                {
                    ty = UnityTypes.UnityParent,
                    value = new UnityComponents
                    {
                        UnityParent = new UnityParent
                        {
                            parent = parentId.ID
                        }
                    }
                };

                count++;
            }

            if (SystemAPI.HasComponent<Unity.Transforms.LocalTransform>(entity))
            {
                arr[count] = new UnityData
                {
                    ty = UnityTypes.UnityTransform,
                    value = new UnityComponents { UnityTransform = SystemAPI.GetComponent<Unity.Transforms.LocalTransform>(entity) }
                };

                count++;
            }


            return (count, true);
        }
    }
}
