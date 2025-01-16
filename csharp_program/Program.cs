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

    [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr async_connect_and_run_query(string uri);

    [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr cass_future_debug_info(IntPtr future);

    [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
    public static extern void cass_future_free_string(IntPtr str);

    public static Task WaitForCassFuture(IntPtr future)
    {
        return Task.Run(async () =>
        {
            while (!cass_future_ready(future))
            {
                Console.WriteLine("Waiting for Rust task to complete...");
                await Task.Yield(); // Yield control to let other tasks run
            }
        });
    }

    static async Task Main(string[] args)
    {
        Console.WriteLine("Calling Rust from C# async code!");

        string uri = "127.0.0.2:9042";
        IntPtr resultPtr = async_connect_and_run_query(uri);

        await WaitForCassFuture(resultPtr);


        IntPtr debugInfoPtr = cass_future_debug_info(resultPtr);
        if (debugInfoPtr != IntPtr.Zero)
        {
            string debugInfo = Marshal.PtrToStringAnsi(debugInfoPtr);
            Console.WriteLine($"Debug Info: {debugInfo}");
            cass_future_free_string(debugInfoPtr); // Free the string after use
        }
    }
}
