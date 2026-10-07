using System;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;

// Evidence from the Windows console screen buffer of the actual CLI process.
public static class ConsoleEvidence
{
    [StructLayout(LayoutKind.Sequential)] public struct Coord { public short X, Y; }
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public short Left, Top, Right, Bottom; }
    [StructLayout(LayoutKind.Sequential)] public struct InfoEx {
        public uint Size;
        public Coord BufferSize, Cursor;
        public ushort Attributes;
        public Rect Window;
        public Coord Maximum;
        public ushort Popup;
        [MarshalAs(UnmanagedType.Bool)] public bool Fullscreen;
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 16)] public uint[] Palette;
    }
    [StructLayout(LayoutKind.Explicit, CharSet = CharSet.Unicode)] public struct Cell {
        [FieldOffset(0)] public char Character;
        [FieldOffset(2)] public ushort Attributes;
    }
    [DllImport("kernel32.dll")] static extern IntPtr GetStdHandle(int id);
    [DllImport("kernel32.dll", SetLastError = true)] static extern bool GetConsoleScreenBufferInfoEx(IntPtr handle, ref InfoEx info);
    [DllImport("kernel32.dll", SetLastError = true)] static extern bool SetConsoleScreenBufferInfoEx(IntPtr handle, ref InfoEx info);
    [DllImport("kernel32.dll", EntryPoint = "ReadConsoleOutputW", CharSet = CharSet.Unicode, SetLastError = true)]
    static extern bool ReadConsoleOutput(IntPtr handle, [Out] Cell[] cells, Coord size, Coord origin, ref Rect area);
    static InfoEx? original;

    static InfoEx GetInfo() {
        var info = new InfoEx { Size = (uint)Marshal.SizeOf<InfoEx>(), Palette = new uint[16] };
        if (!GetConsoleScreenBufferInfoEx(GetStdHandle(-11), ref info)) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
        return info;
    }
    public static void ConfigureTheme(bool light) {
        var info = GetInfo();
        original = info;
        info.Palette = (uint[])info.Palette.Clone();
        // Set the default background/foreground palette entries; SGR reset retains the theme.
        info.Palette[0] = light ? 0xFFFFFFu : 0x000000u;
        info.Palette[7] = light ? 0x000000u : 0xC0C0C0u;
        info.Palette[15] = light ? 0x000000u : 0xFFFFFFu;
        if (!SetConsoleScreenBufferInfoEx(GetStdHandle(-11), ref info)) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
        Console.ForegroundColor = ConsoleColor.Gray;
        Console.BackgroundColor = ConsoleColor.Black;
    }
    public static void RestoreTheme() {
        if (original.HasValue) {
            var info = original.Value;
            if (!SetConsoleScreenBufferInfoEx(GetStdHandle(-11), ref info)) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
        }
    }
    public static void Snapshot(string file) {
        var info = GetInfo();
        var area = info.Window;
        int width = area.Right - area.Left + 1, height = area.Bottom - area.Top + 1;
        var size = new Coord { X = (short)width, Y = (short)height };
        var cells = new Cell[width * height];
        if (!ReadConsoleOutput(GetStdHandle(-11), cells, size, new Coord(), ref area)) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
        var rows = new List<object>();
        for (int y = 0; y < height; y++) {
            var text = new StringBuilder();
            var runs = new List<object>();
            for (int x = 0; x < width;) {
                int start = x;
                ushort attributes = cells[y * width + x].Attributes;
                var segment = new StringBuilder();
                while (x < width && cells[y * width + x].Attributes == attributes) {
                    var cell = cells[y * width + x++];
                    if ((cell.Attributes & 0x0200) == 0) segment.Append(cell.Character == '\0' ? ' ' : cell.Character);
                }
                text.Append(segment);
                runs.Add(new { Column = start, Length = x - start, Attributes = attributes, Foreground = attributes & 15, Background = (attributes >> 4) & 15, Text = segment.ToString() });
            }
            if (text.ToString().Trim().Length > 0) rows.Add(new { Row = y, Text = text.ToString().TrimEnd(), Runs = runs });
        }
        File.WriteAllText(file, JsonSerializer.Serialize(new { Width = width, Height = height, Palette = info.Palette, Rows = rows }), new UTF8Encoding(false));
    }
}
