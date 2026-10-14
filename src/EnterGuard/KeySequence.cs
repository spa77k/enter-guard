namespace EnterGuard;

public readonly record struct InjectedKey(ushort VirtualKey, bool KeyUp = false, bool Extended = false);

// Pure sequence construction permits testing every partial-insertion boundary without Windows.
public static class KeySequence
{
    public const ushort Return = 0x0D, LeftShift = 0xA0, LeftControl = 0xA2, RightControl = 0xA3;

    public static InjectedKey[] Build(EnterAction action, KeyModifiers modifiers, bool keypad)
    {
        if (!modifiers.CanRemap ||
            (action == EnterAction.Newline && modifiers.Control) ||
            (action == EnterAction.Send && !modifiers.Control) ||
            (action != EnterAction.Newline && action != EnterAction.Send))
            throw new ArgumentException("Only an unmodified newline or Ctrl-only send can be remapped.");

        var keys = new List<InjectedKey>(6);
        if (action == EnterAction.Newline) keys.Add(new(LeftShift));
        else
        {
            if (modifiers.LeftControl) keys.Add(new(LeftControl, KeyUp: true));
            if (modifiers.RightControl) keys.Add(new(RightControl, KeyUp: true, Extended: true));
        }
        keys.Add(new(Return, Extended: keypad));
        keys.Add(new(Return, KeyUp: true, Extended: keypad));
        if (action == EnterAction.Newline) keys.Add(new(LeftShift, KeyUp: true));
        else
        {
            if (modifiers.LeftControl) keys.Add(new(LeftControl));
            if (modifiers.RightControl) keys.Add(new(RightControl, Extended: true));
        }
        return keys.ToArray();
    }

    // SendInput inserts in order. Clean up only state changed by the accepted prefix.
    // Never include Enter-down here: an uncertain send must not be retried.
    public static InjectedKey[] Cleanup(InjectedKey[] sequence, uint inserted, KeyModifiers original)
    {
        bool enter = false, shift = false, leftControl = original.LeftControl, rightControl = original.RightControl;
        bool keypad = false;
        for (int i = 0; i < sequence.Length && (uint)i < inserted; i++)
        {
            var key = sequence[i];
            switch (key.VirtualKey)
            {
                case Return: enter = !key.KeyUp; keypad = key.Extended; break;
                case LeftShift: shift = !key.KeyUp; break;
                case LeftControl: leftControl = !key.KeyUp; break;
                case RightControl: rightControl = !key.KeyUp; break;
            }
        }
        var cleanup = new List<InjectedKey>(4);
        if (enter) cleanup.Add(new(Return, KeyUp: true, Extended: keypad));
        if (shift) cleanup.Add(new(LeftShift, KeyUp: true));
        if (original.LeftControl && !leftControl) cleanup.Add(new(LeftControl));
        if (original.RightControl && !rightControl) cleanup.Add(new(RightControl, Extended: true));
        return cleanup.ToArray();
    }
}
