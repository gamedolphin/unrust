using System.Runtime.InteropServices;

namespace unrust.userland
{
    [StructLayout(LayoutKind.Sequential)]
    public struct CustomState
    {
        public const int CustomStateCount = 1;
        public CustomStateType ty;
        public sbyte value;
    }

    public enum CustomStateType: byte
    {
        GameState = 0
    }
}
