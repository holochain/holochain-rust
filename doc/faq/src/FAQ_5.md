# What is Holochain _not_ good for?

You probably should not use Holochain for:

- **Just yourself:** You generally don't need distributed tools to just run something for yourself. The exception would be if you want to run a holochain to synchronize certain data across a bunch of your devices (phone, laptop, desktop, cloud server, etc.)

- **Anonymous, secret, or private data:** Not only do we need to do a security audit of our encryption and permissions, but you're publishing to a shared DHT space, so unless you really know what you're doing, you should not assume data is private. Some time in the future, I'm sure some applications will add an anonymization layer (like TOR), but that is not native.

- **Large files:** Think of holochains more like a database than a file system. Nobody wants to be forced to load and host your big files on their devices just because they are in the neighborhood of its hash. Use something like IPFS if you want a decentralized file system.

- **Data positivist-oriented apps:** If you have built all of your application logic around the idea that data exists as an absolute truth, not as an assertion by an agent at a time, then you would need to rethink your whole approach before putting it in a Holochain app. This is why most existing cryptocurrencies would need significant refactoring to move from blockchain to Holochain, since they are organized around managing the existence of cryptographic tokens.


