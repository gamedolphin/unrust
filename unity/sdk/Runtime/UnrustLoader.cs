using System.Runtime.InteropServices;
using System;
using System.IO;
using UnityEngine;

#if UNITY_EDITOR
using UnityEditor;
#endif

namespace unrust.runtime
{
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public unsafe delegate ContextWrapper* LoadDelegate(LogFunction logger);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public unsafe delegate void InitDelegate(ContextWrapper* ctx, byte* base_path,
                                             CreateFunction creator, UpdateFunction updater, DestroyFunction destroyer);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public unsafe delegate void RegisterDelegate(ContextWrapper* ctx, UnityPrefab prefabs);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public unsafe delegate ulong SpawnDelegate(ContextWrapper* ctx, UnityEntity entity, UnityData* inbuilt, nuint len,
                                               void* custom, nuint custom_len, void* custom_state, nuint custom_state_len);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public unsafe delegate void TickDelegate(ContextWrapper* ctx);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public unsafe delegate void UnloadDelegate(ContextWrapper* ctx);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public unsafe delegate void LogFunction(LogLevel level, byte* str, nuint len);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public unsafe delegate void GameConstructor();

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public unsafe delegate void CreateFunction(EntityData* ptr, nuint len);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public unsafe delegate void UpdateFunction(EntityData* ptr, nuint len);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public unsafe delegate void DestroyFunction(UnityEntity* entity, nuint len);


    public enum LogLevel : byte
    {
        Error = 0,
        Warning = 1,
        Info = 2,
        Debug = 3,
    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe partial struct ContextWrapper
    {
    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe partial struct GameWrapper
    {
    }

    public struct DelegateSet
    {
        public GameConstructor construct;
        public LoadDelegate load;
        public InitDelegate init;
        public RegisterDelegate register;
        public SpawnDelegate spawn;
        public TickDelegate tick;
        public UnloadDelegate unload;
    }

    public class LibraryLoader : IDisposable
    {
        private ILibraryLoader loader;

        public DelegateSet NativeFunctions;

        public LibraryLoader()
        {
#if UNITY_EDITOR_LINUX
            loader = new NativeLoaderEditorLinux();
#endif

            NativeFunctions = loader.Initialize();
        }

        public void Dispose()
        {
            loader.Dispose();
        }
    }

    public interface ILibraryLoader : IDisposable
    {
        public DelegateSet Initialize();
    }

    public static class UnityLogger
    {
        public static unsafe void OnLog(LogLevel level, byte* str, nuint len)
        {
            var msg = System.Text.Encoding.UTF8.GetString(str, (int)len);
            switch (level)
            {
                case LogLevel.Error:
                    UnityEngine.Debug.LogError(msg);
                    break;
                case LogLevel.Warning:
                    UnityEngine.Debug.LogWarning(msg);
                    break;

                case LogLevel.Info:
                    UnityEngine.Debug.Log(msg);
                    break;
            }
        }
    }

#if UNITY_EDITOR_LINUX

    public static class UnrustSettingsEditorFinder
    {
        public static UnrustSettings Get()
        {
            var settings = AssetDatabase.FindAssets("t:UnrustSettings");
            if (settings.Length < 1)
            {
                throw new System.Exception("No unrust settings found!");
            }

            var asset = AssetDatabase.LoadAssetAtPath<UnrustSettings>(AssetDatabase.GUIDToAssetPath(settings[0]));
            return asset;
        }
    }

    public class NativeLoaderEditorLinux : ILibraryLoader
    {
        public IntPtr libraryHandle;

        [DllImport("__Internal")]
        protected static extern IntPtr dlopen(string filename, int flags);

        [DllImport("__Internal")]
        protected static extern IntPtr dlsym(IntPtr handle, string symbol);

        [DllImport("__Internal")]
        public static extern int dlclose(IntPtr handle);

        const int RTLD_NOW = 2; // for dlopen's flags

        private string localPath;

        public static IntPtr OpenLibrary(string path)
        {
            IntPtr handle = dlopen(path, RTLD_NOW);
            if (handle == IntPtr.Zero)
            {
                throw new Exception("Couldn't open native library: " + path);
            }
            return handle;
        }

        public static void CloseLibrary(IntPtr libraryHandle)
        {
            if (dlclose(libraryHandle) != 0)
            {
                UnityEngine.Debug.LogError("failed to unload dynamic library");
            }
        }

        public static T GetDelegate<T>(IntPtr libraryHandle, string functionName) where T : class
        {
            IntPtr symbol = dlsym(libraryHandle, functionName);
            if (symbol == IntPtr.Zero)
            {
                throw new Exception("Couldn't get function: " + functionName);
            }
            return Marshal.GetDelegateForFunctionPointer(symbol, typeof(T)) as T;
        }

        public NativeLoaderEditorLinux()
        {
            var filePath = Path.Combine(Application.dataPath,"unrust");
            string[] files = System.IO.Directory.GetFiles(filePath, "*.so");

            if (files.Length == 0)
            {
                throw new Exception("no compiled files in unrust");
            }

            var libPath = files[0];
            var filename = Path.GetFileNameWithoutExtension(libPath);
            var guid = DateTimeOffset.Now.ToUnixTimeMilliseconds();

            localPath = Path.Combine(UnityEngine.Application.dataPath, $"../Library/{filename}-{guid}.so");

            File.Copy(libPath, localPath);
            libraryHandle = OpenLibrary(localPath);
        }

        public DelegateSet Initialize()
        {
            var construct = GetDelegate<GameConstructor>(libraryHandle, "create_game");
            var load = GetDelegate<LoadDelegate>(libraryHandle, "load");
            var init = GetDelegate<InitDelegate>(libraryHandle, "init");
            var register = GetDelegate<RegisterDelegate>(libraryHandle, "register_prefabs");
            var spawn = GetDelegate<SpawnDelegate>(libraryHandle, "spawn");
            var tick = GetDelegate<TickDelegate>(libraryHandle, "tick");
            var unload = GetDelegate<UnloadDelegate>(libraryHandle, "unload");

            return new DelegateSet
            {
                construct = construct,
                load = load,
                init = init,
                register = register,
                spawn = spawn,
                tick = tick,
                unload = unload,
            };
        }

        ~NativeLoaderEditorLinux()
        {
            if (disposed)
            {
                return;
            }

            Dispose();
        }


        private bool disposed = false;

        public void Dispose()
        {
            if (disposed)
            {
                return;
            }

            disposed = true;
            if (libraryHandle == IntPtr.Zero)
            {
                return;
            }

            CloseLibrary(libraryHandle);
            libraryHandle = IntPtr.Zero;
            File.Delete(localPath);
        }
    }
#endif
}
