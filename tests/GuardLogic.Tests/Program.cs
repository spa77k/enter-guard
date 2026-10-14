using System.Runtime.InteropServices;
using EnterGuard;

int count = 0;
void Check(bool value, string name) { if (!value) throw new Exception(name); count++; }
var none = new KeyModifiers();
var leftCtrl = new KeyModifiers(false, true, false, false, false);
var rightCtrl = new KeyModifiers(false, false, true, false, false);
var bothCtrl = new KeyModifiers(false, true, true, false, false);
var shift = new KeyModifiers(true, false, false, false, false);

// Every enable/foreground/modifier/injection combination, including both Ctrl keys.
foreach (bool enabled in new[] { false, true })
foreach (bool discord in new[] { false, true })
foreach (bool injected in new[] { false, true })
for (int mask = 0; mask < 32; mask++)
{
    var modifiers = new KeyModifiers((mask & 1) != 0, (mask & 2) != 0, (mask & 4) != 0, (mask & 8) != 0, (mask & 16) != 0);
    bool mapped = enabled && discord && modifiers.CanRemap && !injected;
    var initial = mapped ? (modifiers.Control ? EnterAction.Send : EnterAction.Newline) : EnterAction.PassThrough;
    var later = mapped ? EnterAction.Suppress : EnterAction.PassThrough;
    var policy = new EnterPolicy();
    Check(policy.Handle(true, enabled, discord, modifiers, injected) == initial, $"initial {mask}");
    Check(policy.Handle(true, enabled, discord, modifiers, injected) == later, $"repeat {mask}");
    Check(policy.Handle(false, enabled, discord, modifiers, injected) == later, $"release {mask}");
}

var p = new EnterPolicy();
Check(p.Handle(true, true, true, none, false) == EnterAction.Newline, "plain Enter produces newline");
Check(p.Handle(true, true, true, leftCtrl, false) == EnterAction.Suppress, "adding Ctrl mid-hold cannot send");
Check(p.Handle(true, false, false, shift, false) == EnterAction.Suppress, "mapped repeat stays consumed after focus or toggle");
Check(p.Handle(false, false, false, none, false) == EnterAction.Suppress, "mapped release stays consumed");
Check(p.Handle(false, true, true, none, false) == EnterAction.PassThrough, "orphan release passes");
Check(p.Handle(true, true, true, leftCtrl, false) == EnterAction.Send, "fresh Ctrl press sends once");
Check(p.Handle(true, true, true, none, false) == EnterAction.Suppress, "Ctrl release while held cannot insert newline");
Check(p.Handle(false, true, true, none, false) == EnterAction.Suppress, "send original up consumed");

// A passed physical press never becomes a new action, including send, mid-hold.
foreach (var initialModifiers in new[] { shift, leftCtrl, none })
foreach (var laterModifiers in new[] { none, leftCtrl, rightCtrl, bothCtrl })
{
    var policy = new EnterPolicy();
    Check(policy.Handle(true, true, false, initialModifiers, false) == EnterAction.PassThrough, "outside original passes");
    Check(policy.Handle(true, true, true, laterModifiers, false) == EnterAction.Suppress, "unsafe repeat never remaps");
    Check(policy.Handle(true, false, false, shift, false) == EnterAction.Suppress, "repeat suppression latches");
    Check(policy.Handle(false, true, true, laterModifiers, false) == EnterAction.PassThrough, "passed original receives release");
}
var paused = new EnterPolicy();
Check(paused.Handle(true, false, true, none, false) == EnterAction.PassThrough, "paused original passes");
Check(paused.Handle(true, true, true, leftCtrl, false) == EnterAction.Suppress, "enabling midpress cannot send");
Check(paused.Handle(false, true, true, leftCtrl, false) == EnterAction.PassThrough, "paused original release passes");

var primary = new EnterPolicy();
var keypad = new EnterPolicy();
Check(primary.Handle(true, true, true, none, false) == EnterAction.Newline, "main newline");
Check(keypad.Handle(true, true, true, leftCtrl, false) == EnterAction.Send, "independent keypad sends");
Check(keypad.Handle(false, true, true, none, false) == EnterAction.Suppress, "keypad original release consumed");
Check(primary.Handle(true, true, true, none, false) == EnterAction.Suppress, "keypad release does not reset main");
Check(primary.Handle(false, true, true, none, false) == EnterAction.Suppress, "main release consumed");

var injection = new EnterPolicy();
Check(injection.Handle(true, true, true, none, false) == EnterAction.Newline, "physical original maps");
Check(injection.Handle(true, true, true, leftCtrl, true) == EnterAction.PassThrough, "injected down is ignored");
Check(injection.Handle(false, true, true, none, true) == EnterAction.PassThrough, "injected up is ignored");
Check(injection.Handle(true, true, true, none, false) == EnterAction.Suppress, "injected up did not reset physical state");
Check(injection.Handle(false, true, true, none, false) == EnterAction.Suppress, "physical release consumed");

foreach (bool isKeypad in new[] { false, true })
{
    var newline = KeySequence.Build(EnterAction.Newline, none, isKeypad);
    Check(newline.SequenceEqual(new InjectedKey[] {
        new(KeySequence.LeftShift), new(KeySequence.Return, Extended: isKeypad),
        new(KeySequence.Return, KeyUp: true, Extended: isKeypad), new(KeySequence.LeftShift, KeyUp: true)
    }), "newline is exactly Shift down, Enter down/up, Shift up");

    foreach (var modifiers in new[] { none, leftCtrl, rightCtrl, bothCtrl })
    {
        var action = modifiers.Control ? EnterAction.Send : EnterAction.Newline;
        var sequence = KeySequence.Build(action, modifiers, isKeypad);
        Check(sequence.Count(k => k.VirtualKey == KeySequence.Return && !k.KeyUp) == 1, "exactly one Enter down per action");
        Check(sequence.Count(k => k.VirtualKey == KeySequence.Return && k.KeyUp) == 1, "exactly one Enter up per action");
        if (modifiers.Control)
        {
            int enterIndex = Array.FindIndex(sequence, k => k.VirtualKey == KeySequence.Return);
            Check(sequence.Take(enterIndex).All(k => k.KeyUp), "all held Ctrl releases precede Enter");
            Check(sequence.Skip(enterIndex + 2).All(k => !k.KeyUp), "all Ctrl restorations follow Enter up");
            Check(sequence.Count(k => k.VirtualKey == KeySequence.LeftControl) == (modifiers.LeftControl ? 2 : 0), "touch only held left Ctrl");
            Check(sequence.Count(k => k.VirtualKey == KeySequence.RightControl) == (modifiers.RightControl ? 2 : 0), "touch only held right Ctrl");
        }
        Check(sequence.Where(k => k.VirtualKey == KeySequence.RightControl).All(k => k.Extended), "right Ctrl always extended");
        Check(sequence.Where(k => k.VirtualKey == KeySequence.Return).All(k => k.Extended == isKeypad), "Enter identity retained");
        // Every possible accepted prefix must be recoverable without Enter-down.
        for (uint prefix = 0; prefix <= sequence.Length; prefix++)
        {
            var cleanup = KeySequence.Cleanup(sequence, prefix, modifiers);
            Check(cleanup.All(k => k.VirtualKey != KeySequence.Return || k.KeyUp), "cleanup never retries Enter down");
            bool enter = false, temporaryShift = false, left = modifiers.LeftControl, right = modifiers.RightControl;
            foreach (var key in sequence.Take((int)prefix).Concat(cleanup))
            {
                switch (key.VirtualKey)
                {
                    case KeySequence.Return: enter = !key.KeyUp; break;
                    case KeySequence.LeftShift: temporaryShift = !key.KeyUp; break;
                    case KeySequence.LeftControl: left = !key.KeyUp; break;
                    case KeySequence.RightControl: right = !key.KeyUp; break;
                    default: throw new Exception("Unexpected injected key");
                }
            }
            Check(!enter && !temporaryShift && left == modifiers.LeftControl && right == modifiers.RightControl, "partial cleanup restores original states");
            if (prefix == 0 || prefix == sequence.Length) Check(cleanup.Length == 0, "zero/full insertion needs no cleanup");
        }
        var encoded = KeyboardInput.Encode(sequence);
        Check(encoded.Length == sequence.Length, "encoding length");
        for (int i = 0; i < encoded.Length; i++)
        {
            Check(encoded[i].Type == 1, "keyboard input type");
            Check(encoded[i].Data.Keyboard.VirtualKey == sequence[i].VirtualKey, "encoded virtual key");
            Check(encoded[i].Data.Keyboard.Flags == ((sequence[i].KeyUp ? 2u : 0u) | (sequence[i].Extended ? 1u : 0u)), "encoded flags");
            Check(encoded[i].Data.Keyboard.ExtraInfo == KeyboardInput.InjectionTag, "every injection has recursion tag");
        }
    }
}
Check(Marshal.SizeOf<KeyboardInput.Input>() == (IntPtr.Size == 8 ? 40 : 28), "native INPUT size");
Check(Marshal.OffsetOf<KeyboardInput.Input>(nameof(KeyboardInput.Input.Data)).ToInt32() == (IntPtr.Size == 8 ? 8 : 4), "native INPUT union offset");
Check(Marshal.SizeOf<KeyboardInput.KeyboardData>() == (IntPtr.Size == 8 ? 24 : 16), "native KEYBDINPUT size");
Console.WriteLine($"PASS: {count} assertions (pure policy, sequences, every partial cleanup prefix, native layout). Windows UI/Discord/IME runtime not tested.");
