using System;
using System.Runtime.InteropServices;

class Program
{
    [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr rust_hello_world(string uri);

    [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr cass_rust_hello_world();

    [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
    public static extern void cass_future_wait(IntPtr future);

    [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
    public static extern bool cass_future_ready(IntPtr future);

    public static Task WaitForCassFuture(IntPtr future)
    {
        return Task.Run(async () =>
        {
            cass_future_wait(future);
            // while (!cass_future_ready(future))
            // {
            //     Console.WriteLine("Waiting for Rust task to complete...");
            //     await Task.Delay(500); // Poll every 500ms
            // }
        });
    }

    static async Task Main(string[] args)
    {
        Console.WriteLine("Calling Rust from C# async code!");
        IntPtr resultPtr1 = cass_rust_hello_world();

        Console.WriteLine("Calling Rust from C#!");
         // Call the Rust function with a sample URI
        string uri = "127.0.0.2:9042";
        IntPtr resultPtr = rust_hello_world(uri);

        // Convert the returned pointer to a managed string
        string result = Marshal.PtrToStringAnsi(resultPtr);

        // Print the result
        Console.WriteLine($"Result from Rust: {result}");

        await WaitForCassFuture(resultPtr1);
    }
}
