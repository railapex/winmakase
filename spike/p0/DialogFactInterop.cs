using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;

namespace Winmakase.P0
{
    public sealed class NativeRect
    {
        public int x { get; set; }
        public int y { get; set; }
        public int width { get; set; }
        public int height { get; set; }
    }

    public sealed class DialogWindowSnapshot
    {
        public long handle { get; set; }
        public long ownerHandle { get; set; }
        public long rootOwnerHandle { get; set; }
        public int processId { get; set; }
        public long processCreationTimeUtcTicks { get; set; }
        public long windowGeneration { get; set; }
        public int threadId { get; set; }
        public string title { get; set; }
        public string className { get; set; }
        public string styleHex { get; set; }
        public string extendedStyleHex { get; set; }
        public bool visible { get; set; }
        public bool enabled { get; set; }
        public bool hasOwner { get; set; }
        public bool hasCaption { get; set; }
        public bool hasDialogFrame { get; set; }
        public bool hasThickFrame { get; set; }
        public bool isResizable { get; set; }
        public bool isChild { get; set; }
        public bool isPopup { get; set; }
        public bool isDisabled { get; set; }
        public bool isToolWindow { get; set; }
        public bool isAppWindow { get; set; }
        public bool isNoActivate { get; set; }
        public NativeRect rect { get; set; }
    }

    public static class DialogFactReader
    {
        private const int GWL_STYLE = -16;
        private const int GWL_EXSTYLE = -20;
        private const uint GW_OWNER = 4;
        private const uint GA_ROOTOWNER = 3;
        private const uint PROCESS_QUERY_LIMITED_INFORMATION = 0x1000;
        private const string LifetimePropertyName = "Winmakase.P0.DialogLifetime.v1";

        private const long WS_DISABLED = 0x08000000L;
        private const long WS_CAPTION = 0x00C00000L;
        private const long WS_DLGFRAME = 0x00400000L;
        private const long WS_THICKFRAME = 0x00040000L;
        private const long WS_CHILD = 0x40000000L;
        private const long WS_POPUP = unchecked((long)0x80000000UL);
        private const long WS_EX_DLGMODALFRAME = 0x00000001L;
        private const long WS_EX_TOOLWINDOW = 0x00000080L;
        private const long WS_EX_APPWINDOW = 0x00040000L;
        private const long WS_EX_NOACTIVATE = 0x08000000L;

        [StructLayout(LayoutKind.Sequential)]
        private struct RECT
        {
            public int Left;
            public int Top;
            public int Right;
            public int Bottom;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct FILETIME
        {
            public uint LowDateTime;
            public uint HighDateTime;
        }

        private sealed class LifetimeIdentity
        {
            public int ProcessId;
            public long ProcessCreationTimeUtcTicks;
            public long WindowGeneration;
        }

        [DllImport("user32.dll")]
        private static extern bool IsWindow(IntPtr handle);

        [DllImport("user32.dll")]
        private static extern bool IsWindowVisible(IntPtr handle);

        [DllImport("user32.dll")]
        private static extern bool IsWindowEnabled(IntPtr handle);

        [DllImport("user32.dll", SetLastError = true)]
        private static extern uint GetWindowThreadProcessId(IntPtr handle, out uint processId);

        [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern int GetWindowTextW(IntPtr handle, StringBuilder text, int maximumCount);

        [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern int GetClassNameW(IntPtr handle, StringBuilder className, int maximumCount);

        [DllImport("user32.dll", EntryPoint = "GetWindowLongPtrW", SetLastError = true)]
        private static extern IntPtr GetWindowLongPtrW(IntPtr handle, int index);

        [DllImport("user32.dll", SetLastError = true)]
        private static extern bool GetWindowRect(IntPtr handle, out RECT rect);

        [DllImport("user32.dll")]
        private static extern IntPtr GetWindow(IntPtr handle, uint command);

        [DllImport("user32.dll")]
        private static extern IntPtr GetAncestor(IntPtr handle, uint flags);

        [DllImport("user32.dll", CharSet = CharSet.Unicode)]
        private static extern IntPtr GetPropW(IntPtr handle, string propertyName);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern IntPtr OpenProcess(uint desiredAccess, bool inheritHandle, uint processId);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool GetProcessTimes(
            IntPtr process,
            out FILETIME creationTime,
            out FILETIME exitTime,
            out FILETIME kernelTime,
            out FILETIME userTime);

        [DllImport("kernel32.dll")]
        private static extern bool CloseHandle(IntPtr handle);

        public static DialogWindowSnapshot Snapshot(
            long rawHandle,
            int expectedProcessId,
            long expectedProcessCreationTimeUtcTicks,
            long expectedWindowGeneration)
        {
            IntPtr handle = new IntPtr(rawHandle);
            if (!IsWindow(handle))
            {
                throw new InvalidOperationException("Registered fixture HWND is no longer valid.");
            }
            LifetimeIdentity identity = ReadAndValidateLifetime(
                handle,
                expectedProcessId,
                expectedProcessCreationTimeUtcTicks,
                expectedWindowGeneration);
            uint ignoredProcessId;
            uint threadId = GetWindowThreadProcessId(handle, out ignoredProcessId);
            if (threadId == 0)
            {
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not read fixture HWND thread identity.");
            }

            StringBuilder title = new StringBuilder(512);
            GetWindowTextW(handle, title, title.Capacity);
            StringBuilder className = new StringBuilder(256);
            if (GetClassNameW(handle, className, className.Capacity) == 0)
            {
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not read fixture HWND class.");
            }

            long style = GetWindowLongPtrW(handle, GWL_STYLE).ToInt64();
            long extendedStyle = GetWindowLongPtrW(handle, GWL_EXSTYLE).ToInt64();
            RECT rect;
            if (!GetWindowRect(handle, out rect))
            {
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not read fixture HWND rectangle.");
            }

            IntPtr owner = GetWindow(handle, GW_OWNER);
            IntPtr rootOwner = GetAncestor(handle, GA_ROOTOWNER);
            DialogWindowSnapshot snapshot = new DialogWindowSnapshot
            {
                handle = rawHandle,
                ownerHandle = owner.ToInt64(),
                rootOwnerHandle = rootOwner.ToInt64(),
                processId = identity.ProcessId,
                processCreationTimeUtcTicks = identity.ProcessCreationTimeUtcTicks,
                windowGeneration = identity.WindowGeneration,
                threadId = unchecked((int)threadId),
                title = title.ToString(),
                className = className.ToString(),
                styleHex = ToHex(style),
                extendedStyleHex = ToHex(extendedStyle),
                visible = IsWindowVisible(handle),
                enabled = IsWindowEnabled(handle),
                hasOwner = owner != IntPtr.Zero,
                hasCaption = HasFlag(style, WS_CAPTION),
                hasDialogFrame = HasFlag(style, WS_DLGFRAME) ||
                    HasFlag(extendedStyle, WS_EX_DLGMODALFRAME),
                hasThickFrame = HasFlag(style, WS_THICKFRAME),
                isResizable = HasFlag(style, WS_THICKFRAME),
                isChild = HasFlag(style, WS_CHILD),
                isPopup = HasFlag(style, WS_POPUP),
                isDisabled = HasFlag(style, WS_DISABLED),
                isToolWindow = HasFlag(extendedStyle, WS_EX_TOOLWINDOW),
                isAppWindow = HasFlag(extendedStyle, WS_EX_APPWINDOW),
                isNoActivate = HasFlag(extendedStyle, WS_EX_NOACTIVATE),
                rect = new NativeRect
                {
                    x = rect.Left,
                    y = rect.Top,
                    width = rect.Right - rect.Left,
                    height = rect.Bottom - rect.Top
                }
            };

            // Close the collection race: the same process/lifetime property
            // must still own the HWND after every other native fact is read.
            ReadAndValidateLifetime(
                handle,
                expectedProcessId,
                expectedProcessCreationTimeUtcTicks,
                expectedWindowGeneration);
            return snapshot;
        }

        private static LifetimeIdentity ReadAndValidateLifetime(
            IntPtr handle,
            int expectedProcessId,
            long expectedProcessCreationTimeUtcTicks,
            long expectedWindowGeneration)
        {
            if (!IsWindow(handle))
            {
                throw new InvalidOperationException("Registered fixture HWND is no longer valid.");
            }
            uint processId;
            if (GetWindowThreadProcessId(handle, out processId) == 0)
            {
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not read fixture HWND process identity.");
            }
            if (processId != unchecked((uint)expectedProcessId))
            {
                throw new InvalidOperationException("Registered fixture HWND belongs to a different process.");
            }
            if (expectedProcessCreationTimeUtcTicks <= 0)
            {
                throw new InvalidOperationException("Registered fixture process creation identity is invalid.");
            }

            long creationTimeUtcTicks = ReadProcessCreationTimeUtcTicks(processId);
            if (creationTimeUtcTicks != expectedProcessCreationTimeUtcTicks)
            {
                throw new InvalidOperationException("Registered fixture PID has a different process creation identity.");
            }

            long generation = GetPropW(handle, LifetimePropertyName).ToInt64();
            if (expectedWindowGeneration <= 0 || generation != expectedWindowGeneration)
            {
                throw new InvalidOperationException("Registered fixture HWND has a different window lifetime identity.");
            }
            return new LifetimeIdentity
            {
                ProcessId = unchecked((int)processId),
                ProcessCreationTimeUtcTicks = creationTimeUtcTicks,
                WindowGeneration = generation
            };
        }

        private static long ReadProcessCreationTimeUtcTicks(uint processId)
        {
            IntPtr process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, processId);
            if (process == IntPtr.Zero)
            {
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not open the fixture process.");
            }
            try
            {
                FILETIME creation;
                FILETIME exit;
                FILETIME kernel;
                FILETIME user;
                if (!GetProcessTimes(process, out creation, out exit, out kernel, out user))
                {
                    throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not read fixture process creation time.");
                }
                long fileTime = unchecked((long)(((ulong)creation.HighDateTime << 32) | creation.LowDateTime));
                return DateTime.FromFileTimeUtc(fileTime).Ticks;
            }
            finally
            {
                CloseHandle(process);
            }
        }

        private static bool HasFlag(long value, long flag)
        {
            return (value & flag) == flag;
        }

        private static string ToHex(long value)
        {
            return "0x" + unchecked((ulong)value).ToString("X16");
        }
    }
}
