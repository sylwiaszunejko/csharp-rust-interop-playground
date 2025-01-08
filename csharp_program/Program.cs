using System;
using System.Runtime.InteropServices;

class Program
{
    [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr rust_hello_world(string uri);

    static void Main(string[] args)
    {
        Console.WriteLine("Calling Rust from C#!");
         // Call the Rust function with a sample URI
        string uri = "127.0.0.2:9042";
        IntPtr resultPtr = rust_hello_world(uri);

        // Convert the returned pointer to a managed string
        string result = Marshal.PtrToStringAnsi(resultPtr);

        // Print the result
        Console.WriteLine($"Result from Rust: {result}");
    }
}
