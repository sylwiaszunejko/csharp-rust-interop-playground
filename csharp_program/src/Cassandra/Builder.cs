using System.Net;

namespace Cassandra
{
    public class Builder : IInitializer
    {
        private readonly List<object> _contactPoints = new List<object>();
        private bool _addedContactPoints;

        public ICollection<IPEndPoint> ContactPoints
        {
            get { return _contactPoints.Select(c => c as IPEndPoint).Where(c => c != null).ToList(); }
        }

        public Builder AddContactPoint(string address)
        {
            return AddSingleContactPointInternal(address);
        }

        public Builder AddContactPoint(IPAddress address)
        {
            // Avoid creating IPEndPoint entries using the current port,
            // as the user might provide a different one by calling WithPort() after this call
            return AddSingleContactPointInternal(address);
        }

        public Builder AddContactPoint(IPEndPoint address)
        {
            return AddSingleContactPointInternal(address);
        }

        public Cluster Build()
        {
            return Cluster.BuildFrom(this, _contactPoints.Where(c => !(c is IPEndPoint)).ToList());
        }

        private Builder AddSingleContactPointInternal(object contactPoint)
        {
            if (contactPoint == null)
            {
                throw new ArgumentNullException(nameof(contactPoint));
            }

            _addedContactPoints = true;
            _contactPoints.Add(contactPoint);
            return this;
        }
    }
}