// Experimental UIA metadata fixture, NOT a Discord detector and NOT linked into
// the release executable. Never requests Name, Value, TextPattern text or keys.
using System.Windows;
using System.Windows.Automation;
using System.Windows.Controls;
using System.Windows.Interop;

internal enum Composition { Unknown, Active, Clear }
internal record Observation(bool VerifiedChat, bool Password, bool Dialog, Composition Ime, int Generation);
internal static class Gate
{
    // Passing an Enter unchanged is NOT protection from Discord's default send.
    public static bool CanTransform(Observation snapshot, int currentGeneration) =>
        snapshot.VerifiedChat && !snapshot.Password && !snapshot.Dialog &&
        snapshot.Ime == Composition.Clear && snapshot.Generation == currentGeneration;
}
internal static class Program
{
    [STAThread]
    private static int Main()
    {
        foreach (var knownChat in new[] { false, true })
        foreach (var password in new[] { false, true })
        foreach (var dialog in new[] { false, true })
        foreach (var ime in Enum.GetValues<Composition>())
        foreach (var current in new[] { 1, 2 })
        {
            var observation = new Observation(knownChat, password, dialog, ime, 1);
            Check(Gate.CanTransform(observation, current) ==
                (knownChat && !password && !dialog && ime == Composition.Clear && current == 1), "gate matrix");
        }
        var app = new Application();
        var panel = new StackPanel();
        var chat = new TextBox();
        var search = new TextBox();
        var passwordBox = new PasswordBox();
        AutomationProperties.SetAutomationId(chat, "fixture.chat");
        AutomationProperties.SetAutomationId(search, "fixture.search");
        AutomationProperties.SetAutomationId(passwordBox, "fixture.password");
        panel.Children.Add(chat);
        panel.Children.Add(search);
        panel.Children.Add(passwordBox);
        var window = new Window { Title = "EnterGuard isolated test fixture", Content = panel, Width = 400, Height = 250 };
        int result = 1;
        window.ContentRendered += async (_, _) =>
        {
            // Cross-process-style UIA APIs run on MTA, not the window/hook thread.
            nint handle = new WindowInteropHelper(window).Handle;
            try
            {
                await Task.Run(() =>
                {
                    var root = AutomationElement.FromHandle(handle);
                    AutomationElement Find(string id) => root.FindFirst(TreeScope.Descendants,
                        new PropertyCondition(AutomationElement.AutomationIdProperty, id))
                        ?? throw new Exception("Fixture element missing");
                    var chatElement = Find("fixture.chat");
                    var searchElement = Find("fixture.search");
                    var passwordElement = Find("fixture.password");
                    Check(chatElement.Current.ControlType == ControlType.Edit, "chat Edit metadata");
                    Check(searchElement.Current.ControlType == ControlType.Edit, "search also Edit: type alone is insufficient");
                    Check(passwordElement.Current.IsPassword, "password metadata");
                    Check(chatElement.Current.ProcessId == Environment.ProcessId, "own fixture process");
                    Check(!Gate.CanTransform(new(true, false, false, Composition.Unknown, 1), 1), "unsupported composition must not authorize");
                    Check(!Gate.CanTransform(new(true, false, false, Composition.Clear, 1), 2), "stale focus must not authorize");
                });
                Console.WriteLine("PASS: 48 gate combinations and local UIA metadata fixture. Discord and real IME are NOT tested.");
                result = 0;
            }
            catch (Exception e) { Console.Error.WriteLine(e); }
            finally { window.Close(); }
        };
        app.Run(window);
        return result;
    }
    private static void Check(bool condition, string label)
    {
        if (!condition) throw new Exception("FAIL: " + label);
    }
}
