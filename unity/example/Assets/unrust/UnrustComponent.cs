using System.Runtime.InteropServices;

namespace unrust.userland
{
    [StructLayout(LayoutKind.Sequential)]
    public struct CustomData
    {
        public CustomType ty;
        public CustomComponents value;
    }

    public enum CustomType : byte
    {
        DoRotate = 0,
    }

    [StructLayout(LayoutKind.Explicit)]
    public struct CustomComponents
    {
        public const int ComponentCount = 1;
        [FieldOffset(0)] public DoRotate DoRotate;
    }
}
