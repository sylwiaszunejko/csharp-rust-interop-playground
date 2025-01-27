using System.Runtime.InteropServices;
using Cassandra.SessionManagement;

namespace Cassandra
{
    public class Session(IntPtr rustSessionId) : IInternalSession
    {
        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        private static extern bool session_future_ready(IntPtr session);

        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr create_session(IntPtr str, IntPtr id);

        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        private static extern bool session_future_free(IntPtr session);

        private IntPtr rustSessionID = rustSessionId;


        public static Task WaitForCassFuture(IntPtr future)
        {
            return Task.Run(async () =>
            {
                while (!session_future_ready(future))
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