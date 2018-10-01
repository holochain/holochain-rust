# What happens to data when a node leaves the network?

> The DHT of a Holochain app makes sure that there are always enough nodes on the network that hold a given datum.

When people running Holochain apps turn off their device, they leave the network. What happens to their data and the data of other people they were storing? There are always enough nodes that hold a given piece of data in the network so as to prevent data loss when nodes leave. The DHT and Holochain gossip protocol are designed this way. Also, the redundancy factor of data on a given DHT is configurable so it can be fine-tuned for any purpose. For example, a chat app for a small team might set a redundancy factor of 100% in order to prevent long loading times, while an app with thousands of users might have a very small redundancy factor.

