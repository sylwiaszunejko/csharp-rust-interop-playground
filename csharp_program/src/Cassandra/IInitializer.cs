using System.Net;

namespace Cassandra
{
    public interface IInitializer
    {
        ICollection<IPEndPoint> ContactPoints { get; }
    }
}