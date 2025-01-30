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
        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr session_future_get_result(IntPtr session);
        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr execute_query(IntPtr session, string query);
        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        private static extern bool query_future_ready(IntPtr query);


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

        private static Task WaitForQueryFuture(IntPtr future)
        {
            return Task.Run(async () =>
            {
                while (!query_future_ready(future))
                {
                    Console.WriteLine("Waiting for Rust task to complete querying...");
                    await Task.Yield(); // Yield control to let other tasks run
                }
            });
        }

        public static Task<Session> CreateSessionAsync(string uri, string id)
        {
            return Task.Run(async () =>
            {
                IntPtr sessionPtr = create_session(uri, id);
                await WaitForSessionFuture(sessionPtr, id);
                IntPtr resultPtr = session_future_get_result(sessionPtr);
                if (resultPtr == IntPtr.Zero)
                {
                    Console.WriteLine("Session future is not ready or no result.");
                }
                else
                {
                    string errorMessage = Marshal.PtrToStringAnsi(resultPtr);
                    if (!string.IsNullOrEmpty(errorMessage))
                    {
                        Console.WriteLine($"Error occurred: {errorMessage}");
                    }
                    else
                    {
                        // Handle the case where there's no error message (could be a valid result pointer)
                        Console.WriteLine("No error, result processed.");
                    }
                }
                return new Session(resultPtr);
            });
        }

        public Task ExecuteAsync(string statement)
        {
            return Task.Run(async () =>
            {
                Console.WriteLine("Start Executing query... from C#");
                IntPtr resultPtr = execute_query(rustSessionID, statement);
                await WaitForQueryFuture(resultPtr);
                Console.WriteLine("Query executed successfully from C#.");
            });
        }

        public void Dispose()
        {
            session_future_free(rustSessionID);
        }
    }
}