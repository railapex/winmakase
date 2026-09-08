using System;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;
using System.Text;

namespace Winmakase.P0
{
    [ComImport]
    [Guid("00021401-0000-0000-C000-000000000046")]
    [ClassInterface(ClassInterfaceType.None)]
    internal class ShellLink
    {
    }

    [ComImport]
    [Guid("000214F9-0000-0000-C000-000000000046")]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    internal interface IShellLinkW
    {
        void GetPath(
            [Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder file,
            int maxPath,
            out Win32FindData findData,
            uint flags);
        void GetIDList(out IntPtr itemIdList);
        void SetIDList(IntPtr itemIdList);
        void GetDescription([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder name, int maxName);
        void SetDescription([MarshalAs(UnmanagedType.LPWStr)] string name);
        void GetWorkingDirectory([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder directory, int maxPath);
        void SetWorkingDirectory([MarshalAs(UnmanagedType.LPWStr)] string directory);
        void GetArguments([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder arguments, int maxPath);
        void SetArguments([MarshalAs(UnmanagedType.LPWStr)] string arguments);
        void GetHotkey(out short hotkey);
        void SetHotkey(short hotkey);
        void GetShowCmd(out int showCommand);
        void SetShowCmd(int showCommand);
        void GetIconLocation([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder iconPath, int maxPath, out int iconIndex);
        void SetIconLocation([MarshalAs(UnmanagedType.LPWStr)] string iconPath, int iconIndex);
        void SetRelativePath([MarshalAs(UnmanagedType.LPWStr)] string path, uint reserved);
        void Resolve(IntPtr windowHandle, uint flags);
        void SetPath([MarshalAs(UnmanagedType.LPWStr)] string file);
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    internal struct Win32FindData
    {
        public uint fileAttributes;
        public System.Runtime.InteropServices.ComTypes.FILETIME creationTime;
        public System.Runtime.InteropServices.ComTypes.FILETIME lastAccessTime;
        public System.Runtime.InteropServices.ComTypes.FILETIME lastWriteTime;
        public uint fileSizeHigh;
        public uint fileSizeLow;
        public uint reserved0;
        public uint reserved1;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 260)]
        public string fileName;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 14)]
        public string alternateFileName;
    }

    public sealed class ShortcutSnapshot
    {
        public string TargetPath { get; internal set; }
        public string Arguments { get; internal set; }
        public string WorkingDirectory { get; internal set; }
        public string Description { get; internal set; }
    }

    public static class ShortcutFile
    {
        private const int BufferSize = 32768;

        public static void Create(
            string shortcutPath,
            string targetPath,
            string arguments,
            string workingDirectory,
            string description,
            string iconPath)
        {
            IShellLinkW link = (IShellLinkW)new ShellLink();
            try
            {
                link.SetPath(targetPath);
                link.SetArguments(arguments);
                link.SetWorkingDirectory(workingDirectory);
                link.SetDescription(description);
                link.SetIconLocation(iconPath, 0);
                ((IPersistFile)link).Save(shortcutPath, true);
            }
            finally
            {
                Marshal.FinalReleaseComObject(link);
            }
        }

        public static ShortcutSnapshot Read(string shortcutPath)
        {
            IShellLinkW link = (IShellLinkW)new ShellLink();
            try
            {
                ((IPersistFile)link).Load(shortcutPath, 0);
                StringBuilder target = new StringBuilder(BufferSize);
                StringBuilder arguments = new StringBuilder(BufferSize);
                StringBuilder workingDirectory = new StringBuilder(BufferSize);
                StringBuilder description = new StringBuilder(BufferSize);
                Win32FindData findData;
                link.GetPath(target, target.Capacity, out findData, 4);
                link.GetArguments(arguments, arguments.Capacity);
                link.GetWorkingDirectory(workingDirectory, workingDirectory.Capacity);
                link.GetDescription(description, description.Capacity);
                return new ShortcutSnapshot
                {
                    TargetPath = target.ToString(),
                    Arguments = arguments.ToString(),
                    WorkingDirectory = workingDirectory.ToString(),
                    Description = description.ToString()
                };
            }
            finally
            {
                Marshal.FinalReleaseComObject(link);
            }
        }
    }
}
