using System.Diagnostics;
using System.Runtime.InteropServices;

namespace EnterGuard;

internal static class Program
{
    [STAThread]
    static void Main()
    {
        using var mutex = new Mutex(true, "Local\\EnterGuard", out var created);
        if (!created) { MessageBox.Show("すでに起動しています。通知領域のアイコンを確認してください。", "EnterGuard"); return; }
        ApplicationConfiguration.Initialize();
        Application.Run(new GuardWindow());
    }
}

internal sealed class GuardWindow : Form
{
    private readonly EnterPolicy mainEnter = new();
    private readonly EnterPolicy numpadEnter = new();
    private readonly Native.HookProc callback;
    private readonly NotifyIcon tray;
    private readonly Button toggle;
    private readonly Label status;
    private IntPtr hook;
    private bool enabled;
    private bool hotkey;
    private bool failurePending;
    private static uint cachedProcessId;
    private static bool cachedDiscord;
    private static long cachedAt;

    public GuardWindow()
    {
        Text = "EnterGuard — 試用版 0.1.0";
        ClientSize = new Size(620, 520);
        AutoScroll = true;
        MinimumSize = new Size(560, 480);
        StartPosition = FormStartPosition.CenterScreen;
        Font = new Font("Yu Gothic UI", 10);
        using (var stream = typeof(GuardWindow).Assembly.GetManifestResourceStream("EnterGuard.AppIcon.ico"))
        {
            if (stream is not null) { using var embedded = new Icon(stream); Icon = (Icon)embedded.Clone(); }
        }
        var panel = new TableLayoutPanel { Dock = DockStyle.Fill, Padding = new Padding(20), RowCount = 5, ColumnCount = 1, AutoScroll = true };
        panel.RowStyles.Add(new RowStyle(SizeType.AutoSize));
        panel.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
        panel.RowStyles.Add(new RowStyle(SizeType.AutoSize));
        panel.RowStyles.Add(new RowStyle(SizeType.AutoSize));
        panel.RowStyles.Add(new RowStyle(SizeType.AutoSize));
        status = new Label { Text = "停止中：Enterの動作を変更していません", AutoSize = true, Dock = DockStyle.Fill, Font = new Font(Font, FontStyle.Bold) };
        var help = new Label { AutoSize = true, Dock = DockStyle.Fill, Padding = new Padding(0, 16, 0, 16), Text =
            "Windows版 Discord（通常版）専用です。\n\n有効中は、Discordが最前面のとき、Enterを改行（Shift+Enter）、Ctrl+Enterを送信（Enter）へ置き換えます。長押しは1回だけ処理します。\n\n入力欄を識別しないため、検索・ダイアログ・通話画面にも作用します。Ctrl+Enterの本来の通話応答なども置き換わります。Shift / Alt / Windowsキーを含む組み合わせは変更しません。\n\n注意：日本語IMEの変換確定にも作用します。IME使用中の動作は未検証で、誤送信を防ぐ保証はありません。変換確定が必要なときは一時停止してください。\n\nWindows実機では未検証です。大切な会話で使う前に、自分だけのテスト用チャンネルで確認してください。" };
        toggle = new Button { Text = "注意事項を確認して有効にする", AutoSize = true, Dock = DockStyle.Fill, Padding = new Padding(5) };
        toggle.Click += (_, _) => SetEnabled(!enabled);
        var hide = new Button { Text = "通知領域にしまう", AutoSize = true, Dock = DockStyle.Fill };
        hide.Click += (_, _) => Hide();
        var exit = new Button { Text = "終了する", AutoSize = true, Dock = DockStyle.Fill };
        exit.Click += (_, _) => Close();
        panel.Controls.Add(status); panel.Controls.Add(help); panel.Controls.Add(toggle); panel.Controls.Add(hide); panel.Controls.Add(exit); Controls.Add(panel);
        var menu = new ContextMenuStrip();
        menu.Items.Add("画面を開く", null, (_, _) => { Show(); Activate(); });
        menu.Items.Add("有効／一時停止", null, (_, _) => SetEnabled(!enabled));
        menu.Items.Add("終了", null, (_, _) => Close());
        tray = new NotifyIcon { Icon = Icon, Text = "EnterGuard：停止中", ContextMenuStrip = menu, Visible = true };
        tray.DoubleClick += (_, _) => { Show(); Activate(); };
        callback = OnKey;
        hook = Native.SetWindowsHookEx(13, callback, Native.GetModuleHandle(null), 0);
        if (hook == IntPtr.Zero)
        {
            toggle.Enabled = false;
            status.Text = "キーボード監視を開始できません。置き換えは無効です。";
        }
    }
    protected override void OnHandleCreated(EventArgs e)
    {
        base.OnHandleCreated(e);
        hotkey = Native.RegisterHotKey(Handle, 1, 0x0001 | 0x0002 | 0x4000, 0x79); // Ctrl+Alt+F10, no repeat
    }
    protected override void WndProc(ref Message m)
    {
        if (m.Msg == 0x0312 && m.WParam == (IntPtr)1) SetEnabled(!enabled);
        base.WndProc(ref m);
    }
    private void SetEnabled(bool value)
    {
        if (hook == IntPtr.Zero) return;
        enabled = value;
        status.Text = value ? "有効：Enterで改行／Ctrl+Enterで送信" : "停止中：Enterの動作を変更していません";
        if (hotkey) status.Text += "\nCtrl+Alt+F10 で切り替え";
        toggle.Text = value ? "一時停止する" : "注意事項を確認して有効にする";
        tray.Text = value ? "EnterGuard：有効" : "EnterGuard：停止中";
    }
    private IntPtr OnKey(int code, IntPtr message, IntPtr data)
    {
        if (code >= 0)
        {
            var key = Marshal.PtrToStructure<Native.KeyboardData>(data);
            if (key.VkCode == 0x0D && (key.Flags & 0x10) == 0 && key.ExtraInfo != KeyboardInput.InjectionTag)
            {
                bool down = message == (IntPtr)0x100 || message == (IntPtr)0x104;
                bool up = message == (IntPtr)0x101 || message == (IntPtr)0x105;
                if (down || up)
                {
                    var modifiers = down ? ReadModifiers() : default;
                    bool keypad = (key.Flags & 1) != 0;
                    var policy = keypad ? numpadEnter : mainEnter;
                    IntPtr foreground = down && enabled && modifiers.CanRemap ? Native.GetForegroundWindow() : IntPtr.Zero;
                    var action = policy.Handle(down, enabled, foreground != IntPtr.Zero && IsDiscordWindow(foreground), modifiers, false);
                    if (action == EnterAction.Newline || action == EnterAction.Send)
                        Remap(action, modifiers, keypad, foreground);
                    if (action != EnterAction.PassThrough) return (IntPtr)1;
                }
            }
        }
        return Native.CallNextHookEx(hook, code, message, data);
    }
    private void Remap(EnterAction action, KeyModifiers modifiers, bool keypad, IntPtr expectedForeground)
    {
        var sequence = KeySequence.Build(action, modifiers, keypad);
        var inputs = KeyboardInput.Encode(sequence);
        // Check after all preparation and immediately before the single insertion batch.
        // A focus change after this check cannot be ruled out by SendInput.
        if (expectedForeground == IntPtr.Zero || Native.GetForegroundWindow() != expectedForeground)
        {
            DisableAfterFailure("前面ウィンドウが変わったため停止しました。入力は再送していません。");
            return;
        }
        uint inserted = KeyboardInput.Send(inputs);
        if (inserted == inputs.Length) return;

        int error = Marshal.GetLastWin32Error();
        var cleanup = KeyboardInput.Encode(KeySequence.Cleanup(sequence, inserted, modifiers));
        bool restored = cleanup.Length == 0 || KeyboardInput.Send(cleanup) == cleanup.Length;
        // Never retry the Enter action. Even a partial insertion may already have sent it.
        DisableAfterFailure($"入力の置き換えに失敗したため停止しました（{inserted}/{inputs.Length}、エラー{error}）。\n送信済みの可能性があります。再入力の前にDiscordを確認してください。" +
            (restored ? "" : "\nキー状態の復元も確認できません。入力を止め、キー状態が戻らない場合はWindowsを再起動してください。"));
    }
    private void DisableAfterFailure(string reason)
    {
        enabled = false;
        if (failurePending || IsDisposed || !IsHandleCreated) return;
        failurePending = true;
        // Do not manipulate controls or show a modal dialog from the low-level hook.
        BeginInvoke((Action)(() =>
        {
            failurePending = false;
            if (IsDisposed) return;
            SetEnabled(false);
            status.Text = reason;
            tray.Text = "EnterGuard：エラーで停止";
            Show();
        }));
    }
    private static bool Pressed(int key) => (Native.GetAsyncKeyState(key) & 0x8000) != 0;
    private static KeyModifiers ReadModifiers() => new(Pressed(0x10), Pressed(0xA2), Pressed(0xA3), Pressed(0x12), Pressed(0x5B) || Pressed(0x5C));
    private static bool IsDiscordWindow(IntPtr window)
    {
        Native.GetWindowThreadProcessId(window, out uint id);
        if (id == 0) return false;
        long now = Environment.TickCount64;
        if (id == cachedProcessId && now - cachedAt < 500) return cachedDiscord;
        try
        {
            using var process = Process.GetProcessById((int)id);
            bool matches = string.Equals(process.ProcessName, "Discord", StringComparison.OrdinalIgnoreCase);
            cachedProcessId = id; cachedDiscord = matches; cachedAt = now;
            return matches;
        }
        catch (ArgumentException) { return false; }
        catch (System.ComponentModel.Win32Exception) { return false; }
        catch (InvalidOperationException) { return false; }
    }
    protected override void Dispose(bool disposing)
    {
        if (hook != IntPtr.Zero) { Native.UnhookWindowsHookEx(hook); hook = IntPtr.Zero; }
        if (hotkey && IsHandleCreated) Native.UnregisterHotKey(Handle, 1);
        if (disposing) { tray.Visible = false; tray.Dispose(); }
        base.Dispose(disposing);
    }
}
internal static class Native
{
    internal delegate IntPtr HookProc(int code, IntPtr message, IntPtr data);
    [StructLayout(LayoutKind.Sequential)] internal struct KeyboardData { public uint VkCode, ScanCode, Flags, Time; public UIntPtr ExtraInfo; }
    [DllImport("user32.dll", SetLastError = true)] internal static extern IntPtr SetWindowsHookEx(int id, HookProc callback, IntPtr module, uint threadId);
    [DllImport("user32.dll")] [return: MarshalAs(UnmanagedType.Bool)] internal static extern bool UnhookWindowsHookEx(IntPtr hook);
    [DllImport("user32.dll")] internal static extern IntPtr CallNextHookEx(IntPtr hook, int code, IntPtr message, IntPtr data);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode)] internal static extern IntPtr GetModuleHandle(string? name);
    [DllImport("user32.dll")] internal static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] internal static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
    [DllImport("user32.dll")] internal static extern short GetAsyncKeyState(int key);
    [DllImport("user32.dll")] [return: MarshalAs(UnmanagedType.Bool)] internal static extern bool RegisterHotKey(IntPtr window, int id, uint modifiers, uint key);
    [DllImport("user32.dll")] [return: MarshalAs(UnmanagedType.Bool)] internal static extern bool UnregisterHotKey(IntPtr window, int id);
}
