<!-- TOC depthFrom:2 depthTo:6 withLinks:1 updateOnSave:1 orderedList:0 -->

- [Developer Portal](http://developer.holochain.org)
- [Overview](#overview)
- [Shared Data Integrity](#shared-data-integrity)
- [Beyond Blockchain Bottlenecks](#beyond-blockchain-bottlenecks)
- [Summary of Holochain Architecture](#summary-of-holochain-architecture)
	- [Application](#application)
	- [Local Source Chain](#local-source-chain)
	- [Shared Storage on Validating DHT](#shared-storage-on-validating-dht)
- [When to Use Holochain](#when-to-use-holochain)
- [When NOT to Use Holochain](#when-not-to-use-holochain)
- [Why Ceptr? Where does this come from?](#why-ceptr-where-does-this-come-from)
- [A Note to End Users](#a-note-to-end-users)

<!-- /TOC -->

<br />
<div class="alert alert-warning" role="alert">
   <b>Stage of Development:</b> Active development for proof-of-concept stage.
</div>

## Overview
**Holographic storage for distributed applications.** A holochain is a validating distributed hash table (DHT) where every node enforces validation rules on data against the signed chains where the data originated.

In other words, a holochain functions very much **like a blockchain without bottlenecks** when it comes to enforcing validation rules, but is designed to be fully distributed through sharding so each node only needs to hold a portion of the data instead of a full copy of a global ledger. This makes it feasible to run blockchain-like applications on devices as lightweight as mobile phones

## Shared Data Integrity
Historically, data integrity has been ensured by restricting access to data. If we wanted to prevent anybody from tampering with data, we locked it behind firewalls, or set strict permissions on databases and file systems. When your data is centrally stored, you typically have the ability to change whatever you want.

If we want to build peer-to-peer systems where we collectively hold data among many parties, we need better strategies for shared data integrity. Many are excited about building these kinds of applications on the blockchain, because they provide a strategy to maintain integrity of data that can be held by many peers without a single central authority.

However, other limitations have become apparent, such as high computational overhead for achieving consensus, and the Pareto Effects of Proof of Work and Proof of Stake which steer the system toward being more centralized than many would want.

Breakthroughs in shared data integrity enable new social, political, and organizational patterns with less tendencies toward corruption that emerge from power imbalances involved with selective parties controlling data, information, and protocols.

## Beyond Blockchain Bottlenecks
We believe Holochains are one of these breakthroughs, because they take a different approach to ensuring the integrity of shared data. Instead of being built on top of cryptographic tokens they are organized around cryptographic validation of people (peers) validated against an immutable cryptographic record of those peers actions.

This change allows us to manage data integrity without the massive overhead of computing consensus on a global ledger. Our monotonic, validating, graph DHT (distributed hash table) achieves eventual consistency while only allowing valid data to propagate and holding everyone accountable for their actions.

The lower overhead of this approach makes it feasible to run full nodes on devices like cell phones or tablets which don't have massive computing power.

Holochain is designed as a data integrity engine for distributed applications. Unlike a distributed database, there are no methods for users to directly interact with the data because this would bypass application specific validation rules. All interactions happen only through the application code which enforce whatever business rules, application logic, or restrictions they need to, since different applications have different demands for strictness.

## Summary of Holochain Architecture
You can think of a holochain as a shared DHT in which each piece of data is rooted in the signed hash chain of one or more parties. It is a validating DHT so data cannot propagate without first being validated by shared validation rules held by every node -- like every cell in your body has a copy of the same DNA.

Each holochain has these THREE MAIN SUB-SYSTEMS.

![Holochain_Sub-Systems](http://ceptr.org/images/Holochain_Subsystems.png)
### Application
The application is the glue that holds all the parts together into a unified whole. You connect to it with a web browser for a user interface. This application can read and write on your own local signed hash chain, and it can also get data from the Shared DHT, and put data you author out on that shared DHT.

Most importantly, it provides the validation rules which everyone runs to make sure the data being held in the shared DHT can't be tampered with, counterfeited, or lost. As of April 2018, you can write applications in JavaScript or Lisp.

### Local Source Chain
Instead of a shared global ledger like blockchains, every person has their own local chain that they sign things to before publishing them to the shared DHT. Interactions involving multiple parties (such as a currency transfer between two people) are signed by EACH party and committed to their own chains, and then shared to DHT by each party.

![Holochain_Source](http://ceptr.org/images/Holochain_Source.png)

Many of the applications people dream of running in shared decentralized manner (like a distributed Facebook, Twitter, Slack, Uber, or AirBnB) shouldn't need any kind of consensus from a large group of people. Why should I need consensus for a tweet or a social network update? Why should we need consensus for me to reserve your spare room? What do these things have to do with anybody else's agreement?

Thankfully, if an app like this runs on a holochain, I can just write my tweet to my own chain, then share it. Or we can both sign the B&B reservation to each of our chains. And then the information that we've taken this action can propagate over the shared DHT, where nodes can confirm we did this according to shared rules or expectations.

### Shared Storage on Validating DHT
Distributed Hash Tables (DHTs) are already used for file sharing (bittorrent) and other widespread applications. In these systems, the data is content addressable by cryptographic hash, so you can confirm you receive unaltered data by hashing it yourself.

In our validating DHT, we confirm the provenance of every piece of data, validating the signature of its author, and that it has been committed to their local chain. Multi-party transactions create a "crossing" of chains which also assure that even if you try to alter your own chain, your transaction is published by others. Our DHT also has an unusual feature which allows meta-data to be put on data in the DHT which can be used to publish information about a person/node (such as their transactions, or top of their hashchain) or data element (such as tags, comments, or ratings).

Just like validation rules on blockchain nodes, if someone hacked their code to behave differently, even if they colluded with others, the rest of the nodes on the DHT would not validate their altered behavior and they will have essentially just forked themselves out of being able to participate on that holochain.

![Holochain_DHT](http://ceptr.org/images/Holochain_DHT.png)

More details see [Architecture page](Architecture)

## When to Use Holochain
Holochain is designed to support and embody social coherence -- groups that want to collaborate or coordinate together according to a set of agreements which allows them to share data or other value in reliable ways.

Holochains are ideal for:
 * **Social Networks, Social Media & VRM:** You want to run a social network without a company like Facebook in the middle. You want to share, post, publish, or tweet to shared space, while automatically keeping a copy of these things on your own device.
 * **Supply Chains & Open Value Networks:** You want to have information that crosses the boundaries of companies, organizations, countries, which is collaboratively shared and managed, but not under the central control of any one of those organizations.
 * **Cooperatives and New Commons:** You want to create something which is truly held collectively and not by any particular individual. This is especially good for digital assets.
 * **P2P Platforms:** Peer-to-Peer applications where every person has similar capabilities, access, responsibilities, and value is produced collectively.
 * **Collective Intelligence:** Governance, decision-making frameworks, feedback systems, ratings, currencies, annotations, or work flow systems.
 * **Collaborative Applications:** Chats, Discussion Boards, Scheduling Apps, Wikis, Documentation, etc.
 * **Reputational, or Mutual Credit Cryptocurrencies:** Currencies where issuance can be accounted for by actions of peers (like ratings), or through double-entry accounting are well-suited for holochains. Fiat currencies where tokens are thought to exist independent of accountability by agents are more challenging to implement on holochains.


## When NOT to Use Holochain
You probably SHOULD NOT use holochain for:
 * **Just for yourself:** You generally don't need distributed tools to just run something for yourself. The exception would be if you want to run a holochain to synchronize certain data across a bunch of your devices (phone, laptop, desktop, cloud server, etc.)
 * **Anonymous / Secret / Private data:** Not only do we need to do a security audit of our encryption and permissions, but you're publishing to a shared DHT space, so unless you really know what you're doing, you should not assume data is private. Some time in the future, I'm sure some applications will add an anonymization layer (like TOR), but that is not native.
 * **Large files:** Think of holochains more like a database than a file system. Nobody wants to be forced to load and host your big files on their devices just because they are in the neighborhood of its hash. Use something like [IPFS](http://ipfs.io) if you want a decentralized file system.
 * **Data positivist-oriented apps:** If you have built all of your application logic around the idea that data exists as an absolute truth, not as an assertion by an agent at a time, then you would need to rethink your whole approach before putting it in a Holochain app. This is why _most existing cryptocurrencies would need significant refactoring_ to move from blockchain to holochain, since they are organized around managing the existence of cryptographic tokens.

## Why Ceptr? Where does this come from?
Holochain is a part of a much larger vision for distributed computing to enable quantum leaps in our collective intelligence and abilities for groups to organize themselves on all scales. You can find out more about [Ceptr here](http://ceptr.org).

## A Note to End Users
Coming soon there will be applications built to make it easy to use holochains as your distributed database for all your daily needs. Hopefully, these applications will be as easy to find, install, and use as any other software you can think of. However, at the moment, these apps don't exist and holochain is largely for developers trying to build these things for you. Check back in Q2 of 2017 for some cool applications.

For now, please [enjoy our FAQ](http://developer.holochain.net/FAQ). :)
