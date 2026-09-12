using System;
using System.Collections.Generic;
using System.Drawing;
using System.IO;
using System.Text;
using System.Web.Script.Serialization;
using System.Windows.Forms;

internal sealed class FixtureManifest
{
    public int schemaVersion { get; set; }
    public string executableRelativePath { get; set; }
    public string captureDirectoryRelativePath { get; set; }
    public Dictionary<string, string> fixedTitles { get; set; }
    public FixtureAction[] actions { get; set; }
}

internal sealed class FixtureAction
{
    public string id { get; set; }
    public string shortcutFile { get; set; }
    public string[] arguments { get; set; }
    public ExpectedCapture expectedCapture { get; set; }
}

internal sealed class ExpectedCapture
{
    public string fixtureId { get; set; }
    public string window { get; set; }
    public string profileLabel { get; set; }
}

internal sealed class LaunchCapture
{
    public int schemaVersion { get; set; }
    public string executableName { get; set; }
    public string fixtureId { get; set; }
    public string window { get; set; }
    public string profileLabel { get; set; }
    public string[] arguments { get; set; }
}

internal sealed class WindowRegistrationCapture
{
    public int schemaVersion { get; set; }
    public string fixtureId { get; set; }
    public int processId { get; set; }
    public string[] arguments { get; set; }
    public RegisteredFixtureWindow[] windows { get; set; }
}

internal sealed class RegisteredFixtureWindow
{
    public string role { get; set; }
    public string title { get; set; }
    public long handle { get; set; }
    public IntendedWindowFacts intended { get; set; }
}

internal sealed class IntendedWindowFacts
{
    public string classification { get; set; }
    public string ownerRole { get; set; }
    public bool modal { get; set; }
    public bool resizable { get; set; }
    public bool showInTaskbar { get; set; }
    public string formBorderStyle { get; set; }
}

internal static class Program
{
    private const string GuestUser = "WDAGUtilityAccount";
    private const string GuestConsent = "DISPOSABLE-WINDOWS-GUEST";
    private const string MarkerName = ".winmakase-p0-fixture-root";
    private const string MarkerValue = "winmakase-p0-disposable-fixture-v1";
    private static readonly object RegistrationLock = new object();
    private static readonly Dictionary<string, RegisteredFixtureWindow> RegisteredWindows =
        new Dictionary<string, RegisteredFixtureWindow>(StringComparer.Ordinal);
    private static readonly Dictionary<string, Form> OpenWindows =
        new Dictionary<string, Form>(StringComparer.Ordinal);

    [STAThread]
    private static int Main(string[] args)
    {
        string fixtureRoot = Path.GetFullPath(Path.Combine(AppDomain.CurrentDomain.BaseDirectory, ".."));
        if (!IsDisposableGuest(fixtureRoot))
        {
            return 41;
        }

        FixtureManifest manifest;
        FixtureAction action;
        if (!TryLoadAction(fixtureRoot, args, out manifest, out action))
        {
            return 42;
        }

        WriteCapture(fixtureRoot, manifest, action, args);

        Application.EnableVisualStyles();
        Application.SetCompatibleTextRenderingDefault(false);
        Application.Run(CreateMainWindow(fixtureRoot, manifest, action, args));
        return 0;
    }

    private static bool IsDisposableGuest(string fixtureRoot)
    {
        if (!String.Equals(Environment.UserName, GuestUser, StringComparison.Ordinal))
        {
            return false;
        }

        string expectedRoot = Path.GetFullPath(@"C:\WinmakaseP0").TrimEnd('\\');
        if (!String.Equals(fixtureRoot.TrimEnd('\\'), expectedRoot, StringComparison.OrdinalIgnoreCase))
        {
            return false;
        }

        string markerPath = Path.Combine(fixtureRoot, MarkerName);
        try
        {
            return File.Exists(markerPath) &&
                String.Equals(File.ReadAllText(markerPath).Trim(), MarkerValue, StringComparison.Ordinal);
        }
        catch
        {
            return false;
        }
    }

    private static bool TryLoadAction(
        string fixtureRoot,
        string[] args,
        out FixtureManifest manifest,
        out FixtureAction selectedAction)
    {
        manifest = null;
        selectedAction = null;
        if (args.Length != 8)
        {
            return false;
        }

        Dictionary<string, string> parsed = new Dictionary<string, string>(StringComparer.Ordinal);
        for (int index = 0; index < args.Length; index += 2)
        {
            string option = args[index];
            if (parsed.ContainsKey(option))
            {
                return false;
            }
            if (option != "--guest-consent" && option != "--fixture-id" &&
                option != "--window" && option != "--profile-label")
            {
                return false;
            }
            parsed.Add(option, args[index + 1]);
        }

        if (!parsed.ContainsKey("--guest-consent") ||
            !String.Equals(parsed["--guest-consent"], GuestConsent, StringComparison.Ordinal))
        {
            return false;
        }

        string manifestPath = Path.Combine(fixtureRoot, "fixture-actions.json");
        try
        {
            JavaScriptSerializer serializer = new JavaScriptSerializer();
            manifest = serializer.Deserialize<FixtureManifest>(File.ReadAllText(manifestPath, Encoding.UTF8));
        }
        catch
        {
            return false;
        }

        if (manifest == null || manifest.schemaVersion != 1 || manifest.actions == null ||
            manifest.captureDirectoryRelativePath != "data" || manifest.fixedTitles == null)
        {
            return false;
        }

        foreach (FixtureAction action in manifest.actions)
        {
            if (action != null && String.Equals(action.id, parsed["--fixture-id"], StringComparison.Ordinal))
            {
                selectedAction = action;
                break;
            }
        }

        if (selectedAction == null || selectedAction.expectedCapture == null || selectedAction.arguments == null ||
            selectedAction.arguments.Length != args.Length)
        {
            return false;
        }
        for (int index = 0; index < args.Length; index++)
        {
            if (!String.Equals(selectedAction.arguments[index], args[index], StringComparison.Ordinal))
            {
                return false;
            }
        }

        return String.Equals(selectedAction.expectedCapture.fixtureId, parsed["--fixture-id"], StringComparison.Ordinal) &&
            String.Equals(selectedAction.expectedCapture.window, parsed["--window"], StringComparison.Ordinal) &&
            String.Equals(selectedAction.expectedCapture.profileLabel, parsed["--profile-label"], StringComparison.Ordinal) &&
            parsed["--profile-label"].Length <= 64;
    }

    private static void WriteCapture(
        string fixtureRoot,
        FixtureManifest manifest,
        FixtureAction action,
        string[] args)
    {
        string dataDirectory = Path.Combine(fixtureRoot, manifest.captureDirectoryRelativePath);
        Directory.CreateDirectory(dataDirectory);
        string capturePath = Path.Combine(dataDirectory, action.id + ".json");
        LaunchCapture capture = new LaunchCapture
        {
            schemaVersion = 1,
            executableName = "WinmakaseP0Fixture.exe",
            fixtureId = action.expectedCapture.fixtureId,
            window = action.expectedCapture.window,
            profileLabel = action.expectedCapture.profileLabel,
            arguments = args
        };
        JavaScriptSerializer serializer = new JavaScriptSerializer();
        File.WriteAllText(capturePath, serializer.Serialize(capture), new UTF8Encoding(false));
    }

    private static Form CreateMainWindow(
        string fixtureRoot,
        FixtureManifest manifest,
        FixtureAction action,
        string[] args)
    {
        Form main = new Form
        {
            Text = manifest.fixedTitles["main"],
            Name = "WinmakaseP0Main",
            Size = new Size(720, 420),
            MinimumSize = new Size(480, 300),
            StartPosition = FormStartPosition.CenterScreen,
            FormBorderStyle = FormBorderStyle.Sizable,
            MaximizeBox = true,
            MinimizeBox = true
        };
        main.Shown += delegate
        {
            RegisterWindow(
                fixtureRoot,
                action,
                args,
                main,
                "main",
                "main",
                null,
                false,
                true,
                true);
        };

        Label summary = new Label
        {
            AutoSize = true,
            Location = new Point(24, 24),
            Text = "Fixture: " + action.id + Environment.NewLine +
                   "Profile: " + action.expectedCapture.profileLabel + Environment.NewLine +
                   "Resize this main window; its title remains fixed."
        };
        main.Controls.Add(summary);

        Button ownedButton = MakeButton("Owned dialog", 24, 120);
        ownedButton.Click += delegate
        {
            ShowOwnedDialog(fixtureRoot, action, args, main, manifest, false);
        };
        main.Controls.Add(ownedButton);

        Button modalButton = MakeButton("Modal dialog", 184, 120);
        modalButton.Click += delegate
        {
            ShowOwnedDialog(fixtureRoot, action, args, main, manifest, true);
        };
        main.Controls.Add(modalButton);

        Button utilityButton = MakeButton("Utility popup", 344, 120);
        utilityButton.Click += delegate
        {
            ShowUtility(fixtureRoot, action, args, main, manifest);
        };
        main.Controls.Add(utilityButton);

        main.Shown += delegate
        {
            main.BeginInvoke((MethodInvoker)delegate
            {
                if (action.expectedCapture.window == "owned")
                {
                    ShowOwnedDialog(fixtureRoot, action, args, main, manifest, false);
                }
                else if (action.expectedCapture.window == "modal")
                {
                    ShowOwnedDialog(fixtureRoot, action, args, main, manifest, true);
                }
                else if (action.expectedCapture.window == "utility")
                {
                    ShowUtility(fixtureRoot, action, args, main, manifest);
                }
            });
        };
        return main;
    }

    private static Button MakeButton(string text, int left, int top)
    {
        return new Button
        {
            Text = text,
            Location = new Point(left, top),
            Size = new Size(140, 34)
        };
    }

    private static void ShowOwnedDialog(
        string fixtureRoot,
        FixtureAction action,
        string[] args,
        Form owner,
        FixtureManifest manifest,
        bool modal)
    {
        string role = modal ? "modal" : "owned";
        if (TryActivateOpenWindow(role))
        {
            return;
        }
        Form dialog = new Form
        {
            Text = manifest.fixedTitles[modal ? "modal" : "owned"],
            Name = modal ? "WinmakaseP0ModalDialog" : "WinmakaseP0OwnedDialog",
            ClientSize = new Size(380, 150),
            FormBorderStyle = FormBorderStyle.FixedDialog,
            MaximizeBox = false,
            MinimizeBox = false,
            ShowInTaskbar = false,
            StartPosition = FormStartPosition.CenterParent
        };
        dialog.Shown += delegate
        {
            RegisterWindow(
                fixtureRoot,
                action,
                args,
                dialog,
                role,
                "dialog",
                "main",
                modal,
                false,
                false);
        };
        dialog.FormClosed += delegate
        {
            UnregisterWindow(fixtureRoot, action, args, role, dialog);
        };
        dialog.Controls.Add(new Label
        {
            AutoSize = true,
            Location = new Point(24, 24),
            Text = modal ? "Owned modal dialog. The main window is blocked." :
                "Owned modeless dialog. This window is not resizable."
        });
        Button close = MakeButton("Close", 216, 92);
        close.Click += delegate { dialog.Close(); };
        dialog.Controls.Add(close);

        if (modal)
        {
            dialog.ShowDialog(owner);
            dialog.Dispose();
        }
        else
        {
            dialog.Show(owner);
        }
    }

    private static void ShowUtility(
        string fixtureRoot,
        FixtureAction action,
        string[] args,
        Form owner,
        FixtureManifest manifest)
    {
        if (TryActivateOpenWindow("utility"))
        {
            return;
        }
        Form utility = new Form
        {
            Text = manifest.fixedTitles["utility"],
            Name = "WinmakaseP0Utility",
            ClientSize = new Size(320, 110),
            FormBorderStyle = FormBorderStyle.FixedToolWindow,
            MaximizeBox = false,
            MinimizeBox = false,
            ShowInTaskbar = false,
            StartPosition = FormStartPosition.Manual,
            Location = new Point(owner.Right - 340, owner.Top + 64)
        };
        utility.Shown += delegate
        {
            RegisterWindow(
                fixtureRoot,
                action,
                args,
                utility,
                "utility",
                "utility",
                "main",
                false,
                false,
                false);
        };
        utility.FormClosed += delegate
        {
            UnregisterWindow(fixtureRoot, action, args, "utility", utility);
        };
        utility.Controls.Add(new Label
        {
            AutoSize = true,
            Location = new Point(20, 20),
            Text = "Owned utility/tool popup."
        });
        Button close = MakeButton("Close", 156, 60);
        close.Click += delegate { utility.Close(); };
        utility.Controls.Add(close);
        utility.Show(owner);
    }

    private static void RegisterWindow(
        string fixtureRoot,
        FixtureAction action,
        string[] args,
        Form form,
        string role,
        string classification,
        string ownerRole,
        bool modal,
        bool resizable,
        bool showInTaskbar)
    {
        RegisteredFixtureWindow window = new RegisteredFixtureWindow
        {
            role = role,
            title = form.Text,
            handle = form.Handle.ToInt64(),
            intended = new IntendedWindowFacts
            {
                classification = classification,
                ownerRole = ownerRole,
                modal = modal,
                resizable = resizable,
                showInTaskbar = showInTaskbar,
                formBorderStyle = form.FormBorderStyle.ToString()
            }
        };

        lock (RegistrationLock)
        {
            RegisteredWindows[role] = window;
            OpenWindows[role] = form;
            WriteWindowRegistrations(fixtureRoot, action, args);
        }
    }

    private static void UnregisterWindow(
        string fixtureRoot,
        FixtureAction action,
        string[] args,
        string role,
        Form form)
    {
        lock (RegistrationLock)
        {
            Form openWindow;
            if (!OpenWindows.TryGetValue(role, out openWindow) ||
                !Object.ReferenceEquals(openWindow, form))
            {
                return;
            }
            OpenWindows.Remove(role);
            RegisteredWindows.Remove(role);
            WriteWindowRegistrations(fixtureRoot, action, args);
        }
    }

    private static bool TryActivateOpenWindow(string role)
    {
        Form form;
        if (!OpenWindows.TryGetValue(role, out form) || form.IsDisposed)
        {
            return false;
        }
        form.Activate();
        return true;
    }

    private static void WriteWindowRegistrations(
        string fixtureRoot,
        FixtureAction action,
        string[] args)
    {
        List<RegisteredFixtureWindow> ordered = new List<RegisteredFixtureWindow>();
        foreach (string role in new[] { "main", "owned", "modal", "utility" })
        {
            RegisteredFixtureWindow window;
            if (RegisteredWindows.TryGetValue(role, out window))
            {
                ordered.Add(window);
            }
        }

        WindowRegistrationCapture capture = new WindowRegistrationCapture
        {
            schemaVersion = 1,
            fixtureId = action.id,
            processId = System.Diagnostics.Process.GetCurrentProcess().Id,
            arguments = args,
            windows = ordered.ToArray()
        };
        string path = Path.Combine(fixtureRoot, "data", action.id + "-windows.json");
        JavaScriptSerializer serializer = new JavaScriptSerializer();
        File.WriteAllText(path, serializer.Serialize(capture), new UTF8Encoding(false));
    }
}
