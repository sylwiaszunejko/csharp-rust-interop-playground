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
    public static extern IntPtr async_connect_and_run_query(string uri, string id);

    [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr cass_future_debug_info(IntPtr future);

    [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
    public static extern void cass_future_free_string(IntPtr str);

    public static Task WaitForCassFuture(IntPtr future, string id)
    {
        return Task.Run(async () =>
        {
            while (!cass_future_ready(future))
            {
                Console.WriteLine($"Waiting for Rust task to complete... {id}");
                await Task.Yield(); // Yield control to let other tasks run
            }
        });
    }

    static async Task Main(string[] args)
    {
        Console.WriteLine("Calling Rust from C# async code!");

        string uri = "127.0.0.2:9042";
        string id = "1234";
        IntPtr resultPtr = async_connect_and_run_query(uri, id);

        string id1 = "5678";
        IntPtr resultPtr1 = async_connect_and_run_query(uri, id1);

        await Task.WhenAll(
            WaitForCassFuture(resultPtr, id),
            WaitForCassFuture(resultPtr1, id1)
        );

        IntPtr debugInfoPtr = cass_future_debug_info(resultPtr);
        if (debugInfoPtr != IntPtr.Zero)
        {
            string debugInfo = Marshal.PtrToStringAnsi(debugInfoPtr);
            Console.WriteLine($"Debug Info: {debugInfo}, {id}");
            cass_future_free_string(debugInfoPtr); // Free the string after use
        }

        IntPtr debugInfoPtr1 = cass_future_debug_info(resultPtr1);
        if (debugInfoPtr1 != IntPtr.Zero)
        {
            string debugInfo = Marshal.PtrToStringAnsi(debugInfoPtr1);
            Console.WriteLine($"Debug Info: {debugInfo}, {id1}");
            cass_future_free_string(debugInfoPtr1); // Free the string after use
        }
    }
}
