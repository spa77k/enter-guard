using System.Runtime.InteropServices;

namespace EnterGuard;

internal static class KeyboardInput
{
    // A non-secret tag prevents recursive processing even if another hook alters flags.
    internal static readonly UIntPtr InjectionTag = (UIntPtr)0x44454731u;

    internal static Input[] Encode(InjectedKey[] keys)
    {
        var inputs = new Input[keys.Length];
        for (int i = 0; i < keys.Length; i++)
        {
            var key = keys[i];
            inputs[i] = new Input
            {
                Type = 1, // INPUT_KEYBOARD
                Data = new InputUnion
                {
                    Keyboard = new KeyboardData
                    {
                        VirtualKey = key.VirtualKey,
                        Flags = (key.KeyUp ? 0x0002u : 0) | (key.Extended ? 0x0001u : 0),
                        ExtraInfo = InjectionTag
                    }
                }
            };
        }
        return inputs;
    }

    internal static uint Send(Input[] inputs) => SendInput((uint)inputs.Length, inputs, Marshal.SizeOf<Input>());

    // Include the complete native union. A keyboard-only union would incorrectly make
    // INPUT 32 bytes on x64; Win32 requires 40 bytes (28 on x86).
    [StructLayout(LayoutKind.Sequential)]
    internal struct Input { internal uint Type; internal InputUnion Data; }
    [StructLayout(LayoutKind.Explicit)]
    internal struct InputUnion
    {
        [FieldOffset(0)] internal MouseData Mouse;
        [FieldOffset(0)] internal KeyboardData Keyboard;
        [FieldOffset(0)] internal HardwareData Hardware;
    }
    [StructLayout(LayoutKind.Sequential)]
    internal struct KeyboardData
    {
        internal ushort VirtualKey, ScanCode;
        internal uint Flags, Time;
        internal UIntPtr ExtraInfo;
    }
    [StructLayout(LayoutKind.Sequential)]
    internal struct MouseData
    {
        internal int X, Y;
        internal uint MouseInfo, Flags, Time;
        internal UIntPtr ExtraInfo;
    }
    [StructLayout(LayoutKind.Sequential)]
    internal struct HardwareData { internal uint Message; internal ushort ParamLow, ParamHigh; }

    [DllImport("user32.dll", SetLastError = true)]
    private static extern uint SendInput(uint count, [In] Input[] inputs, int size);
}
