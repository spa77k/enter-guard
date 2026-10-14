namespace EnterGuard;

public enum EnterAction { PassThrough, Suppress, Newline, Send }

public readonly record struct KeyModifiers(bool Shift, bool LeftControl, bool RightControl, bool Alt, bool Windows)
{
    public bool Control => LeftControl || RightControl;
    public bool CanRemap => !Shift && !Alt && !Windows;
}

// One instance per physical Enter key. Only the initial down may create an action.
// A previously passed press can have unsafe repeats suppressed, but is never remapped mid-hold.
public sealed class EnterPolicy
{
    private bool held;
    private bool suppressRepeats;
    private bool suppressRelease;

    public EnterAction Handle(bool down, bool enabled, bool discord, KeyModifiers modifiers, bool injected)
    {
        if (injected) return EnterAction.PassThrough;
        if (!down)
        {
            var result = held && suppressRelease ? EnterAction.Suppress : EnterAction.PassThrough;
            held = suppressRepeats = suppressRelease = false;
            return result;
        }

        bool eligible = enabled && discord && modifiers.CanRemap;
        if (held)
        {
            suppressRepeats |= eligible;
            return suppressRepeats ? EnterAction.Suppress : EnterAction.PassThrough;
        }

        held = true;
        suppressRepeats = suppressRelease = eligible;
        if (!eligible) return EnterAction.PassThrough;
        return modifiers.Control ? EnterAction.Send : EnterAction.Newline;
    }
}
