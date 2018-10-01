# How is Holochain more environmentally ethical than blockchain?

> Holochain removes the need for global consensus, and with it the expenditure of massive amounts of electricity to synchronize millions of nodes about data that aren't relevant to them.

There are two reasons Holochain is vastly more efficient than blockchain and more ethical in a green sense:

1. It eliminates the need for all nodes to be synchronized with each other in global consensus. Sharding is usually enabled on Holochain. This means that when two nodes make a transaction, each node saves a countersigned record of that transaction. Additionally, the transaction is published to the [Distributed Hash Table](https://www.youtube.com/watch?v=FhF_kvgfEZM) (sent to and saved by some unpredictably random nodes that can be looked up later for retrieval).

    Sharding is configurable by app, and in some cases it's a good idea to turn it off. For example, imagine a distributed Slack-like team messaging app. With only 40-50 members, full synchronization would be worth the extra bandwidth requirement for the benefit of offline messages and reduced load times. But for most applications, global synchronization isn't really needed and sharding is kept on.

    Because of DHTs, and the sharding they enable, Holochain actually doesn't rely on the transfer of large amounts of redundant information, and uses vastly less bandwidth than blockchain.

2. There's no mining on Holochain. Blockchain's proof-of-work system provides a hefty incentive for thousands of people to spend the processing power of their CPUs and GPUs using up [huge amounts](https://digiconomist.net/bitcoin-energy-consumption) [of electricity](https://motherboard.vice.com/en_us/article/ywbbpm/bitcoin-mining-electricity-consumption-ethereum-energy-climate-change) on solving a meaningless cryptographic puzzle. Holochain doesn't have mining.

