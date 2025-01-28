using System.Runtime.InteropServices;
using Cassandra.SessionManagement;

namespace Cassandra
{
    public class Session(IntPtr rustSessionId) : IInternalSession
    {
        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        private static extern bool session_future_ready(IntPtr session);

        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr create_session(string str, string id);

        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        private static extern bool session_future_free(IntPtr session);

        private IntPtr rustSessionID = rustSessionId;

        private static Task WaitForSessionFuture(IntPtr future, string id)
        {
            return Task.Run(async () =>
            {
                while (!session_future_ready(future))
                {
                    Console.WriteLine($"Waiting for Rust task to complete... {id}");
                    await Task.Yield(); // Yield control to let other tasks run
                }
            });
        }

        public static Task<Session> CreateSessionAsync(string uri, string id)
        {
            return Task.Run(async () =>
            {
                IntPtr resultPtr = create_session(uri, id);
                await WaitForSessionFuture(resultPtr, id);
                return new Session(resultPtr);
            });
        }

        // public Task ExecuteAsync(IntPtr session_future, string statement)
        // {
        //     return Task.Run(async () =>
        //     {
        //         IntPtr resultPtr = async_run_query(session_future, statement);
        //         await WaitForCassFuture(resultPtr);
        //     });
        // }

        public void Dispose()
        {
            session_future_free(rustSessionID);
        }
    }
}