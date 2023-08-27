using System;
using UnityEngine;
using Unity.Entities;

namespace unrust.runtime
{
    public unsafe delegate ulong CustomCreateCallback(void* custom, nuint custom_len, void* state, nuint state_len);

    public unsafe class NativeWrapper : IDisposable
    {
        public static Func<EntityManager, Entity, CustomCreateCallback, ulong> CustomCreates;

        private LibraryLoader loader;
        private DelegateSet nativeFunctions;

        private LogFunction logger;

        private ContextWrapper* ctx;

        public NativeWrapper()
        {
            this.loader = new LibraryLoader();
            this.nativeFunctions = loader.NativeFunctions;
            this.logger = UnityLogger.OnLog;
            nativeFunctions.construct();
            this.ctx = nativeFunctions.load(this.logger);
        }

        public void Initialize(CreateFunction create, UpdateFunction update, DestroyFunction destroy)
        {
            var base_path = Application.dataPath;
            var base_bytes = System.Text.Encoding.UTF8.GetBytes(base_path + char.MinValue); // add null termination
            fixed (byte* ptr = base_bytes)
            {
                this.nativeFunctions.init(this.ctx, ptr, create, update, destroy);
            }
        }

        public void RegisterPrefabs(UnityPrefab prefab)
        {
            this.nativeFunctions.register(this.ctx, prefab);
        }

        public void Update()
        {
            this.nativeFunctions.tick(this.ctx);
        }

        public ulong Create(EntityManager manager, Entity entity, UnityData* comps, nuint length)
        {
            if (CustomCreates != null)
            {
                return CustomCreates.Invoke(manager, entity, (void* custom, nuint custom_len, void* state, nuint state_len) =>
                {
                    return nativeFunctions.spawn(ctx, entity, comps, length, custom, custom_len, state, state_len);
                });
            }
            else
            {
                return nativeFunctions.spawn(ctx, entity, comps, length, null, 0, null, 0);
            }
        }

        public void Dispose()
        {
            if (this.ctx != null)
            {
                this.nativeFunctions.unload(this.ctx);
                this.ctx = null;
            }

            this.loader.Dispose();
        }
    }

}
