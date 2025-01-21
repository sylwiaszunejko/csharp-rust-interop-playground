using System.Runtime.InteropServices;
using Cassandra.SessionManagement;

namespace Cassandra
{
    public class Session : IInternalSession
    {
        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr async_run_query(IntPtr future, string query);

        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        public static extern bool cass_future_ready(IntPtr future);

        public static Task WaitForCassFuture(IntPtr future)
        {
            return Task.Run(async () =>
            {
                while (!cass_future_ready(future))
                {
                    Console.WriteLine($"Waiting for Rust task to complete...");
                    await Task.Yield(); // Yield control to let other tasks run
                }
            });
        }

        public Task ExecuteAsync(IntPtr session_future, string statement)
        {
            return Task.Run(async () =>
            {
                IntPtr resultPtr = async_run_query(session_future, statement);
                await WaitForCassFuture(resultPtr);
            });
        }

        public void Dispose()
        {
        }
    }
}