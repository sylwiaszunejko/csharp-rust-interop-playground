using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Swift;

namespace Cassandra
{
    public class Cluster
    {
        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr async_connect_and_run_query(string uri, string id);

        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr cass_future_debug_info(IntPtr future);

        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        public static extern void cass_future_free_string(IntPtr str);

        [DllImport("rust_library", CallingConvention = CallingConvention.Cdecl)]
        public static extern bool cass_future_ready(IntPtr future);

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

        private readonly IEnumerable<object> contactPoints;

        private Cluster(IEnumerable<object> contactP)
        {
            contactPoints = contactP;
        }

        public static Builder Builder()
        {
            return new Builder();
        }

        public static Cluster BuildFrom(IInitializer initializer)
        {
            return BuildFrom(initializer, null);
        }

        internal static Cluster BuildFrom(IInitializer initializer, IReadOnlyList<object>? nonIpEndPointContactPoints)
        {
            nonIpEndPointContactPoints = nonIpEndPointContactPoints ?? new object[0];
            if (initializer.ContactPoints.Count == 0 && nonIpEndPointContactPoints.Count == 0)
            {
                throw new ArgumentException("Cannot build a cluster without contact points");
            }

            return new Cluster(
                initializer.ContactPoints.Concat(nonIpEndPointContactPoints));
        }

        // public Task<ISession> ConnectAsync()
        // {
        //     return ConnectAsync("Configuration.ClientOptions.DefaultKeyspace");
        // }

        // public async Task<ISession> ConnectAsync(string keyspace)
        // {
        //     if (contactPoints == null || !contactPoints.Any())
        //     {
        //         throw new InvalidOperationException("Contact points cannot be null or empty.");
        //     }
        //     var firstContactPoint = contactPoints.First();
        //     if (firstContactPoint == null)
        //     {
        //         throw new InvalidOperationException("First contact point cannot be null.");
        //     }
        //     string uri = firstContactPoint?.ToString() ?? throw new InvalidOperationException("First contact point cannot be null.");
        //     string id = "1234";
        //     IntPtr resultPtr = async_connect_and_run_query(uri, id);
        //     await WaitForCassFuture(resultPtr, id);
        //     return new Session();
        // }
    }
}