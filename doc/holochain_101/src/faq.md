# Frequently Asked Questions

1. [How is Holochain different from blockchain?](#how-is-holochain-different-from-blockchain)
2. [Why do you call it "Holochain"?](#why-do-you-call-it-holochain)
3. [How is Holochain different from a DHT (Distributed Hash Table)?](#how-is-holochain-different-from-a-dht-distributed-hash-table)
4. [What kind of projects is Holochain good for?](#what-kind-of-projects-is-holochain-good-for)
 [What is Holochain _not_ good for?](#what-is-holochain-not-good-for)
5. [What is Holochain's consensus algorithm?](#what-is-holochains-consensus-algorithm)
6. [Can you run a cryptocurrency on Holochain?](#can-you-run-a-cryptocurrency-on-holochain)
7. [How is Holochain different from __________?](#how-is-holochain-different-from-__________)
8. [What language is Holochain written in? What languages can I use to make Holochain apps?](#what-language-is-holochain-written-in-what-languages-can-i-use-to-make-holochain-apps)
9. [Is Holochain open source?](#is-holochain-open-source)
10 [How is Holochain more environmentally ethical than blockchain?](#how-is-holochain-more-environmentally-ethical-than-blockchain)
11. [How are data validated on Holochain?](#how-are-data-validated-on-holochain)
12. [What happens to data when a node leaves the network?](#what-happens-to-data-when-a-node-leaves-the-network)
13. [Should I build my coin/token on Holochain?](#should-i-build-my-cointoken-on-holochain)
14. [What does “agent-centric” mean? How is this different from “data-centric”?](#what-does-agent-centric-mean-how-is-this-different-from-data-centric)
15. [What is the TPS (Transactions Per Second) on Holochain?](#what-is-the-tps-transactions-per-second-on-holochain)



## How is Holochain different from blockchain?

> Holochain and blockchain are built for fundamentally different use cases. Blockchain is relatively good for systems where it’s absolutely necessary to maintain global consensus. Holochain is much better than blockchain at anything that requires less than universal consensus (most things): It’s faster, more efficient, more scalable, adaptable, and extendable. 

Long before blockchains were [hash chains](https://en.wikipedia.org/wiki/Hash_chain) and [hash trees](https://en.wikipedia.org/wiki/Merkle_tree). These structures can be used to ensure tamper-proof data integrity as progressive versions or additions to data are made. These kinds of hashes are often used as reference points to ensure data hasn't been messed with—like making sure you're getting the program you meant to download, not some virus in its place.

Instead of trying to manage global consensus for every change to a huge blockchain ledger, every participant has [their own signed hash chain](https://medium.com/metacurrency-project/perspectives-on-blockchains-and-cryptocurrencies-7ef391605bd1#.kmous6d7z) ([countersigned for transactions](https://medium.com/metacurrency-project/beyond-blockchain-simple-scalable-cryptocurrencies-1eb7aebac6ae#.u1idviscz) involving others). After data is signed to local chains, it is shared to a [DHT](https://en.wikipedia.org/wiki/Distributed_hash_table) where every node runs the same validation rules (like blockchain nodes all run the [same validation rules](https://bitcoin.org/en/bitcoin-core/features/validation). If someone breaks those rules, the DHT rejects their data—their chain has forked away from the holochain.

The initial [Bitcoin white paper](https://bitcoin.org/bitcoin.pdf) introduced a blockchain as an architecture for decentralized production of a chain of digital currency transactions. This solved two problems (time/sequence of transactions, and randomizing who writes to the chain) with one main innovation of bundling transactions into blocks which somebody wins the prize of being able to commit to the chain if they [solve a busywork problem](https://en.bitcoin.it/wiki/Hashcash) faster than others.

Now Bitcoin and blockchain have pervaded people's consciousness and many perceive it as a solution for all sorts of decentralized applications. However, when the problems are framed slightly differently, there are much more efficient and elegant solutions (like holochains) which don't have the [processing bottlenecks](https://www.google.com/search?q=blockchain+bottleneck) of global consensus, storage requirements of everyone having a [FULL copy](https://blockchain.info/charts/blocks-size) of all the data, or [wasting so much electricity ](https://blog.p2pfoundation.net/essay-of-the-day-bitcoin-mining-and-its-energy-footprint/2015/12/20) on busywork.

## Why do you call it "Holochain"?

> A variety of reasons: it's a composed whole of other technologies, it's structurally holographic, and it empowers holistic patterns.

### A unified cryptographic _whole_

Holochain is made from multiple cryptographic technologies composed into a new whole.

- **Hashchains:** Hashchains provide immutable data integrity and definitive time sequence from the vantage point of each node. Technically, we're using hash trees—blockchains do too, but they're not called blocktrees, so we're not calling these holotrees.

- **Cryptographic signing** of chains, messages, and validation confirmations maintain authorship, provenance, and accountability. Countersigning of transactions/interactions between multiple parties provide non-repudiation and "locking" of chains.

- **DHT (Distributed Hash Table)** leverages cryptographic hashes for content addressable storage, while randomizing of interactions by hashing into neighborhoods to impede collusion, and processing validation #1 and #2 to store data on the DHT.

### *Holo*graphic storage

Every node has a resilient sample of the whole. Like cutting a hologram, if you were to cut a Holochain network in half (make it so half the nodes were isolated from the other half), you would have two whole, functioning systems, not two partial, broken systems.

This seems to be the strategy used to create resilience in natural systems. For example, where is your DNA stored? Every cell carries its own copy, with different functions expressed based on the role of that cell.

Where is the English language stored? Every speaker carries it. People have different areas of expertise, or exposure to different slang or specialized vocabularies. Nobody has a complete copy, nor is anyone's version exactly the same as anyone else, If you disappeared half of the English speakers, it would not degrade the language much.

If you keep cutting a hologram smaller and smaller eventually the image degrades enough to stop being recognizable, and depending on the resiliency rules for DHT neighborhoods, holochains would likely share a similar fate. Although, if the process of killing off the nodes was not instantaneous, the network may be able to keep reshuffling data per redundancy requirements to keep it alive.

### *Hol*archy

Holochains are composable with each other into new levels of unification. In other words, Holochains can build on decentralized capacities provided by other Holochains, making new holistic patterns possible. Like bodies build new unity on holographic storage patterns that cells use for DNA, and a society build new unity on the holographic storage patterns of language, and so on.

## How is Holochain different from a DHT (Distributed Hash Table)?

DHTs enable key/value pair storage and retrieval across many machines. The only validation rules they have is the hash of the data itself to confirm what you're getting is probably what you intended to get. They have no other means to confirm authenticity, provenance, timelines, or integrity of data sources.

In fact, since many DHTs are used for illegal file sharing (Napster, Bittorrent, Sharezaa, etc.), they are designed to protect anonymity of uploaders so they won't get in trouble. File sharing DHTs frequently serve virus infected files, planted by uploaders trying to infect digital pirates. There's no accountability for actions or reliable way to ensure bad data doesn't spread.

By embedding validation rules as a condition for the propagation of data, our DHT keeps its data bound to signed source chains. This can provide similar consistency and rule enforcement as blockchain ledgers asynchronously so bottlenecks of immediate consensus become a thing of the past.

The DHT leverages the signed source chains to ensure tamper-proof immutability of data, as well as cryptographic signatures to verify its origins and provenance.

The Holochain DHT also emulates aspects of a graph database by enabling people to connect links to other hashes in the DHT tagged with semantic markers. This helps solve the problem of finding the hashes that you want to retrieve from the DHT. For example, if I have the hash of your user identity, I could query it for links to blogs you've published to a holochain so that I can find them without knowing either the hash or the content. This is part of how we eliminate the need for tracking nodes that many DHTs rely on.

## What kind of projects is Holochain good for?

Sharing collaborative data without centralized control. Imagine a completely decentralized Wikipedia, DNS without root servers, or the ability to have fast reliable queries on a fully distributed PKI, etc.

- **Social Networks, Social Media & VRM:** You want to run a social network without a company like Facebook in the middle. You want to share, post, publish, or tweet to shared space, while automatically keeping a copy of these things on your own device.

- **Supply Chains & Open Value Networks:** You want to have information that crosses the boundaries of companies, organizations, countries, which is collaboratively shared and managed, but not under the central control of any one of those organizations.

- **Cooperatives and New Commons:** You want to create something which is truly held collectively and not by any particular individual. This is especially good for digital assets.

- **P2P Platforms:** Peer-to-Peer applications where every person has similar capabilities, access, responsibilities, and value is produced collectively.

- **Collective Intelligence:** Governance, decision-making frameworks, feedback systems, ratings, currencies, annotations, or work flow systems.

- **Collaborative Applications:** Chats, Discussion Boards, Scheduling Apps, Wikis, Documentation, etc.

- **Reputational or Mutual Credit Cryptocurrencies:** Currencies where issuance can be accounted for by actions of peers (like ratings), or through double-entry accounting are well-suited for holochains. Fiat currencies where tokens are thought to exist independent of accountability by agents are more challenging to implement on holochains.

## What is Holochain _not_ good for?

You probably should not use Holochain for:

- **Just yourself:** You generally don't need distributed tools to just run something for yourself. The exception would be if you want to run a holochain to synchronize certain data across a bunch of your devices (phone, laptop, desktop, cloud server, etc.)

- **Anonymous, secret, or private data:** Not only do we need to do a security audit of our encryption and permissions, but you're publishing to a shared DHT space, so unless you really know what you're doing, you should not assume data is private. Some time in the future, I'm sure some applications will add an anonymization layer (like TOR), but that is not native.

- **Large files:** Think of holochains more like a database than a file system. Nobody wants to be forced to load and host your big files on their devices just because they are in the neighborhood of its hash. Use something like IPFS if you want a decentralized file system.

- **Data positivist-oriented apps:** If you have built all of your application logic around the idea that data exists as an absolute truth, not as an assertion by an agent at a time, then you would need to rethink your whole approach before putting it in a Holochain app. This is why most existing cryptocurrencies would need significant refactoring to move from blockchain to Holochain, since they are organized around managing the existence of cryptographic tokens.


## What is Holochain's consensus algorithm?

> Holochains don't manage consensus, at least not about some absolute perspective on data or sequence of events. They manage distributed data integrity. Holochains do rely on consensus about the validation rules (DNA) which define that integrity, but so does every blockchain or blockchain alternative (e.g. Bitcoin Core). If you have different validation rules, you're not on the same chain. These validation rules establish the "data physics," and then applications are built on that foundation.

In making Holochain, our goal is to keep it "as simple as possible, but no simpler" for providing data integrity for fully distributed applications. As we understand it, information integrity does not require consensus about an absolute order of events. You know how we know? Because the real world works this way—meaning, the physically distributed systems outside of computers. Atoms, molecules, cells, bodies each maintain the integrity of their individual and collective state just fine without consensus on a global ledger.

Not only is there no consensus about an absolute order of events, but if you understand the General Theory of Relativity, then you'll understand there is in fact no real sequence of events, only sequences relative to a particular vantage point.

That's how holochains are implemented. Each source chain for each person/agent/participant in a Holochain preserves the immutable data integrity and order of events of that agent's actions from their vantage point. As data is published from a source chain to the validating DHT, then other agents sign their validation, per the shared "physics" encoded into the DNA of that Holochain.

The minor exception to the singular vantage point of each chain, is the case when a multi-party transaction is signed to each party's chain. That is an act of consensus—but consensus on a very small scale—just between the parties involved in the transaction. Each party signs the exact same transaction with links to each of their previous chain entries. Luckily, it's pretty easy to reach consensus between 2 or 3 parties. In fact, that is already why they're doing a transaction together, because they all agree to it.

Holochains do sign every change of data and timestamp (without a universal time synchronization solution). This provides ample foundation for most applications which need solid data integrity for shared data in a fully distributed multi-agent system. Surely, there will be people who will build consensus algorithms on top of that foundation (maybe like rounds, witnesses, supermajorities of [Swirlds](https://www.swirlds.com/)).

However, if your system is designed around data having one absolute true state, not one which is dynamic and varied based on vantage point, we would suggest you rethink your design. So far, for every problem space where people thought they needed an absolute sequence of events or global consensus, we have been able to map an alternate approach without those requirements. Also, we already know this is how the world outside of computers works, so to design your system to require (or construct) an artificial reality is probably setting yourself up for failure, or at the very least for massive amounts of unnecessary computation, communication, and fragility within your system.

## How is Holochain more environmentally ethical than blockchain?

> Holochain removes the need for global consensus, and with it the expenditure of massive amounts of electricity to synchronize millions of nodes about data that aren't relevant to them.

There are two reasons Holochain is vastly more efficient than blockchain and more ethical in a green sense:

1. It eliminates the need for all nodes to be synchronized with each other in global consensus. Sharding is usually enabled on Holochain. This means that when two nodes make a transaction, each node saves a countersigned record of that transaction. Additionally, the transaction is published to the [Distributed Hash Table](https://www.youtube.com/watch?v=FhF_kvgfEZM) (sent to and saved by some unpredictably random nodes that can be looked up later for retrieval).

    Sharding is configurable by app, and in some cases it's a good idea to turn it off. For example, imagine a distributed Slack-like team messaging app. With only 40-50 members, full synchronization would be worth the extra bandwidth requirement for the benefit of offline messages and reduced load times. But for most applications, global synchronization isn't really needed and sharding is kept on.

    Because of DHTs, and the sharding they enable, Holochain actually doesn't rely on the transfer of large amounts of redundant information, and uses vastly less bandwidth than blockchain.

2. There's no mining on Holochain. Blockchain's proof-of-work system provides a hefty incentive for thousands of people to spend the processing power of their CPUs and GPUs using up [huge amounts](https://digiconomist.net/bitcoin-energy-consumption) [of electricity](https://motherboard.vice.com/en_us/article/ywbbpm/bitcoin-mining-electricity-consumption-ethereum-energy-climate-change) on solving a meaningless cryptographic puzzle. Holochain doesn't have mining.

## How is Holochain different from __________?

**TODO: Update with reference to Rust project.**

Please see the [Comparisons page](https://github.com/Holochain/holochain-proto/wiki/Comparisons).

## What language is Holochain written in? What languages can I use to make Holochain apps?

Holochain is written in the Rust programming language. At a low level, Holochain runs WebAssembly code, but for all practical purposes developers will write applications in a language that compiles to WebAssembly such as Rust, C, C++, Go, etc. For now, only Rust has tier 1 support for writing apps, because it has a ["Holochain Development Kit" library](https://github.com/holochain/holochain-rust/tree/develop/hdk-rust) which makes writing WebAssembly apps easy.

## Is Holochain open source?

Yes, it has an open source [license](https://github.com/Holochain/holochain-rust/#license).

## Can you run a cryptocurrency on Holochain?

> Theoretically, yes—but for the moment, we'd discourage it.

If you don't know how to issue currencies through mutual credit, or how to account for them through double entry accounting, then you probably shouldn't build one on Holochain. If you do understand those key principles, than it is not very difficult to build a cryptocurrency for which Holochain provides ample accounting and data integrity.

However, you probably shouldn't try to do it in the way everyone is used to building cryptocurrencies on a global ledger of cryptographic tokens. [Determining the status of tokens/coins](https://en.bitcoin.it/wiki/Double-spending) is what create the need for global consensus (about the existence/status/validity of the token or coin). However, there are [other approaches to making currencies](https://medium.com/metacurrency-project/perspectives-on-blockchains-and-cryptocurrencies-7ef391605bd1) which, for example, involve [issuance via mutual credit](https://medium.com/metacurrency-project/beyond-blockchain-simple-scalable-cryptocurrencies-1eb7aebac6ae) instead of issuance by fiat.

Unfortunately, this is a hotly contested topic by many who often don't have a deep understanding of currency design nor cryptography, so we're not going to go too deep in this FAQ. We intend to publish a white paper on this topic soon, as well as launch some currencies built this way.

## How are data validated on Holochain?

> On Holochain, each node that receives a record of a transaction validates it against the shared application rules and gossips it to their peers. If the rules are broken, that transaction is rejected by the validator.

There is no overall, global "correctness" (or consensus) built in to Holochain. Instead, each node that receives a record of a transaction validates it against the shared application rules and gossips it to their peers. If the rules are broken, that transaction is rejected by the validator. If foul play is detected on a node's part (the node is either propagating or validating bad data) that node is blocked and a warning is sent to others. Here's [an infographic](https://i.imgur.com/bjp7Txg.png) describing this process. In summary, instead of a global consensus system, Holochain uses an [accountability-based system](https://www.youtube.com/watch?v=PVTnEKxwYls&t=1s) with data validation by peers.

Applying this to the example of 'Ourbnb', an imaginary distributed version ofAirbnb: The Ourbnb Holochain app would certainly be written with a rule, "don't rent your apartment to two parties at the same time." So the moment a user rents to two parties at the same time, nodes receiving that datum on the DHT attempt to validate it against the app rules, detect a collision, and reject it. Holochain's gossip protocol is designed to operate at a rate at which collisions will be detected nearly immediately by gossiping peers. And since Holochain doesn't have a coin built into it, it incentivizes users to cooperate and co-create.

As a user, you don't need to trust the provider of the application you're using, only agree with the shared protocols that make up the application itself. Aside from being responsible for the maintenance and security of apps they provide, application providers on Holochain are not like traditional application providers today (think Facebook, Twitter, etc.). They don't host your data because your data is stored by you and a random subset of the users of the application.

## What happens to data when a node leaves the network?

> The DHT of a Holochain app makes sure that there are always enough nodes on the network that hold a given datum.

When people running Holochain apps turn off their device, they leave the network. What happens to their data and the data of other people they were storing? There are always enough nodes that hold a given piece of data in the network so as to prevent data loss when nodes leave. The DHT and Holochain gossip protocol are designed this way. Also, the redundancy factor of data on a given DHT is configurable so it can be fine-tuned for any purpose. For example, a chat app for a small team might set a redundancy factor of 100% in order to prevent long loading times, while an app with thousands of users might have a very small redundancy factor.

## Should I build my coin/token on Holochain?

> Since it's agent-centric instead of data-centric like traditional blockchains, Holochain isn't the best platform on which to build a token or coin.

The idea of tokens or coins is a direct representation of a system being data-centric. While theoretically it would be possible to create a token on Holochain, it would be taking a step back instead of a step forward. The more exciting possibility is creating mutual credit currencies on Holochain. These are agent-centric currencies that are designed to facilitate active exchange of value and flourishing ecosystems instead of hoarding.

## What does “agent-centric” mean? How is this different from “data-centric?”

> Agent-centric systems view data not as an object, but as a shared experience.

Traditional blockchains are data-centric: they rely on and are built around the concept that data is a thing—an object. Holochain transitions to agent-centricism: the idea that data is shared experiences seen from many points of view. It's not a thing. It's a collection of shared, relative experiences. Einstein discovered this about the physical world a hundred years ago—Relativity. So why are modern blockchains that are supposedly "cutting edge" still falling back on this antiquated idea that data is an object, and for two agents to have different views of one piece of data is wrong?

Holochain is deeply agent-centric. Using tech that embodies this mindset enables vastly richer interactions and collaboration to happen through its technology while at the same time being thousands of times more efficient.

## What is the TPS (Transactions Per Second) on Holochain?

> Holochain doesn't have a set TPS (transactions per second) like other blockchain-based or blockchain-derived projects might because there's central point through which all transactions must pass. Instead, Holochain is a generalized protocol for distributed computing.

It's common to ask a blockchain project, "How much can your technology handle? What's its TPS?" This is because nearly all of these projects are built around the limiting idea of a global ledger.

But you are not asking, how many posts per second Facebook can do. Why? Because there is no technical problem, adding more servers to Facebook's data center (only maybe monetary problems).

You are not asking how many emails per second the internet can handle, because there is no single bottleneck for email-sending, like there would be with a centralized approach.

Why are we seeing a transaction limit with blockchain networks? Because blockchain in a strange way marries a decentralized p2p network of nodes with the logical notion of one absolute truth, i.e. the blockchain being one big decentralized database of transactions. It tries to maintain this way of thinking about apps that we are used to from centralized servers. It forces every node into the same "consensus". That is implemented by having everybody share and validate everything. That does work, and maybe there are few usecases (like a global naming system maybe?) where it might be advantageous.. but applying that for everything is nonsensical.

Holochain is not forcing such a model. Instead it allows for building applications that are like email. The application is rather like a protocol, or grammar, or (I prefer this language) like a dance. If you know the dance (If you have a copy of the validation rules of the app) you can tell who else is dancing that dance and who is not. The difference between Holochain and something like email is that (similarly to blockhain) Holochain is applying 1. cryptographic signatures and 2. tamper proof hash-chains (hence Holo*chain*) so that you can build a distributed system you can trust in. You know it is impossible (I'd rather say: very very hard) to game somebody. This so far was only possible by having trusted authorities like banks or Facebook.

So, Holochain as an app framework does not pose any limit of transactions per second because there is no place where all transactions have to go through. It is like asking, "how many words can humanity speak per second?" Well, with every human being born, that number increases. Same for Holochain.
