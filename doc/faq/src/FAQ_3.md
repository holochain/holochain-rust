# How is Holochain different from a DHT (Distributed Hash Table)?

DHTs enable key/value pair storage and retrieval across many machines. The only validation rules they have is the hash of the data itself to confirm what you're getting is probably what you intended to get. They have no other means to confirm authenticity, provenance, timelines, or integrity of data sources.

In fact, since many DHTs are used for illegal file sharing (Napster, Bittorrent, Sharezaa, etc.), they are designed to protect anonymity of uploaders so they won't get in trouble. File sharing DHTs frequently serve virus infected files, planted by uploaders trying to infect digital pirates. There's no accountability for actions or reliable way to ensure bad data doesn't spread.

By embedding validation rules as a condition for the propagation of data, our DHT keeps its data bound to signed source chains. This can provide similar consistency and rule enforcement as blockchain ledgers asynchronously so bottlenecks of immediate consensus become of the thing of the past.

The DHT leverages the signed source chains to ensure tamper-proof immutability of data, as well as cryptographic signatures to verify its origins and provenance.

The Holochain DHT also emulates aspects of a graph database by enabling people to connect links to other hashes in the DHT tagged with semantic markers. This helps solve the problem of finding the hashes that you want to retrieve from the DHT. For example, if I have the hash of your user identity, I could query it for links to blogs you've published to a holochain so that I can find them without knowing either the hash or the content. This is part of how we eliminate the need for tracking nodes that many DHTs rely on.

