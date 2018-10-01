# `this page is a draft!`

**DISCLAIMER:** This page compares Holochain to various other decentralized data protocols and systems. _We can't guarantee the accuracy of claims made on this page about projects or software other than Holochain._ The purpose of this page is to further your understanding Holochain by comparing it to other projects you might already be familiar with, not to teach you about those other projects. To learn about those other projects, please review their resources instead; links to their websites and white papers are provided.

#

### Comparisons

* [Hashgraph](#hashgraph)
* [IOTA](#iota)
* [Substatum](#substratum)
* [IPFS](#ipfs)
* [ECSA](#ecsa)
* [Urbit](#urbit)
* [Radix](#radix)
* [Byteball](#byteball)
* [DAT Project](#dat-project)
* [DADI](#dadi)
* [Todachain](#todachain)
* [Scuttlebutt](#scuttlebutt)
* [Maidsafe](#maidsafe)
* [Blockstack](#blockstack)
* [EOS](#eos)
* [ZeroNet](#zeronet)
* [Skycoin](#skycoin)

***

## Holochain itself

| [Website](https://holochain.org) |
|:-|
| [**White paper**](https://github.com/Holochain/holochain-proto/blob/develop/holochain.pdf) |
| [**Codebase**](https://github.com/Holochain/holochain-proto) |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
| ✔ | ✔ | ✔ | ✔ | ✔ | ✔ |

Holochain is a framework for building fully decentralized, peer-to-peer apps. In its architecture, Holochain leans away from the severely limiting and often destructive consensus-based and data-centric practices made popular by blockchain technologies. Holochain is unique in that it utilizes DHTs for collective data storage and proliferation while maintaining agent-centric data integrity via personal hash chains held by each node.

* Holochain is not a blockchain. On a blockchain, every node on a network maintains the same state of the entire network. On Holochain, each node maintains its own history in a personal, cryptographically tamper-proof chain.
* DHTs (Distributed Hash Tables) are implemented on Holochain to create shared public space. Each node carries some of the shared data so that if a node goes offline, its data isn't lost to the community. This is configurable for each app's use-case.
* Distributed validation built in to Holochain means that every user of an app agrees to that app's validation rules. If these rules are broken, other nodes can tell how and by whom, and then react accordingly.
* Each Holochain app is its own network.

[Holochain Explained (video)](https://www.youtube.com/watch?v=hyCtYrHJebs)

Holochain is released in a working Alpha state with a suite of working prototype and proof-of-concept apps.

## Hashgraph

| [Website](http://hashgraph.com)  |
|:-|
| [**White paper**](https://www.hederahashgraph.com/whitepaper)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
| ✔ | ✔ | ✔ | ✔ | ✔ | ✘ |


##### Matt's answer from Quora:
An important point is that different tools are appropriate for different jobs. In the world of “generating global consensus,” I see hashgraph as a big jump forward from blockchain.

Holochain is trying to solve a different problem. We are not trying to generate “global consensus.”

We are creating a broader architecture for facilitating social coordination. Our aim is truly distributed applications - applications that work at massive scale AND scope - and that place the individual user, not the network, at the center. We envision a world where individuals are able to fluidly bridge across applications, reading content out of one app and writing it into another. In this way, “sense-making” begins to move beyond the boundaries of a single “digital space and its digital rules” and becomes much more fluid and organic.

Our design has been guided by a goal of enabling application ecosystems that adapt in response to the real lived experiences of individual users and of individual groups. We seek to enable appropriate social coherence (from the perspective of the participants) rather than global consensus.

One of the primary advantages that a distributed system has is the fact that different actors see the world from different perspectives. In the natural world, this diversity of viewpoints is actually what makes ecosystems thrive. It makes delegation profitable, and is a requirement for regenerative systems.

From our perspective, the cost of global consensus is that it loses this diversity - it is achieved by transforming a rich variety of perspective into a mono-perspective. There are instances where that choice may be a useful one, but for most coordination problems, global consensus feels like a solution that creates more problems than it solves.

##### Will's additions:
In short, Holochain is open source and agent centric, and Hashgraph is proprietary and data-centric. Hashgraph seems to be trying to do distributed global-consensus but do it much faster. With Holochain, we've skirted the limitations of global consensus by using a faster and more adaptable system.

Like blockchain, Hashgraph is a method of generating global consensus. We believe global consensus is neither necessary nor desirable in most use cases and so have designed Holochain as a tool for the vast majority of human interactions that require less than global consensus. But more importantly, Holochain wasn’t designed to merely record a history of transactions, it was designed to create a broader infrastructure for facilitating social coordination via powerful, truly distributed apps. (You could say the same for IOTA, Stellar, and similar projects.) Also, Holochain is open source and Hashgraph is not.

##### Art's answer:

Hashgraph is one of the closest innovations to us that I've seen for people shifting from the blockchain mindset -- but there are a few gaps that I see (from my completely biased perspective).

Notice that all their examples show the agents and who is doing what (A B C D E), so in the shift from data-centric blockchain toward agent-centric holochain, they are hybridizing.

This is what enables them to create a consensus algorithm based on gossip about gossip, because it is looking at things from the perspective of EACH AGENT and then they somewhat arbitrarily say the median time something has been seen shall be its official time.

So, they have made the partial mindshift from data-centric to agent-centric, and it is possible (likely even) that if we exposed all of our gossip data to the app level in holochain an app could do its own variant of hashgraph consensus (except that they patented it).

But my questions are... why always the focus on consensus? Why only one reality? Why only one arbitrary formula for manufacturing that consensus? Aren’t there many settings where no consensus is needed? Or where it would be valuable to engage in a social process around disagreement?

In holochain, you have implicit consensus when a data element saturates a majority of the DHT neighborhood where that data element resides. A later attempt to PUT that data to the DHT will produce a collision. But what if it is okay to have the collision, and just say "Okay, two people have now invented the Calculus." or whatever. So now you have two authors, with different timestamps, and histories, and so what?

Well the "so what" comes into play when the data is a rival resource -- like a Twitter handle, a Domain Name, or a cryptocoin. Then you want to handle the collision differently and block the later addition telling them the name is already taken. For general computing on distributed apps, this covers 99.9% of use cases. And the way we implement currencies using agent-centric crypto-accounting instead of data-centric coins, this case never happens.

So the only time you have to worry about consensus is if the collision happens BEFORE the DHT neighborhood got saturated with one implicit consensus. So if two people try to register the name at the same-ish time, how do you resolve it?

Well, why should we pretend there's an absolute median time answer, rather than let the app builder decide.
 - Maybe you start an auction and it goes to the highest bidder.
 - Maybe you look at their reputation for community contribution and let the greatest contributor have it.
 - Maybe you send them each a message to resolve the conflict with each other.
 - Maybe you vote on it.

The point is, that for the very small percentage of times you would have this kind of collision, why would you want to swallow the computational overhead on ALL OTHER non-colliding bits of data, and rule out the possibility of context appropriate "consensus" solutions by hard-coding in only one arbitrary approach.

If 99.9% of data in distributed apps is non-rival, or non-conflicting, shouldn't we just trigger the special consensus resolution on that .1% of the cases and bear the (computational or social) cost of that overhead only on those cases?

Since Blockchain grew up with it’s ONE APP being managing rival coins, everyone thinks consensus is at the heart of the matter. I would assert this also why blockchain doesn’t scale for doing generalized computation for dApps. They can barely scale coin transactions which are kind of a ridiculously simple app.

## IOTA

| [Website](https://iota.org/)  |
|:-|
| [**White paper**](https://iota.readme.io/docs/whitepaper)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
| ✔ | ✔ | ✘ | ✔ | ✔ | ✘ |

IOTA (like many of the blockchain alternatives) is still limited to a transaction ledger. It does not do generalized computing or run distributed applications. It has one app - tokenization - and instead of having a single chain of blocks that everyone agrees on, it uses a more chaotic pattern, called the tangle, to ensure multiple nodes/people are still validating most transactions. The tangle is a DAG (Directed Acyclic Graph) instead of a chain, which still links hashes, but has multiple possible paths through its history. This allows IOTA to reduce some of the bottlenecks of having a single global ledger, because there are multiple "tops" to the tangle that you can connect your transaction(s) to. When enough others follow your path through to your new transactions, and build new ones on top of those, they will be considered validated and become a part of the the tangle's history.

Anyway... I don't think they've solved the problem of general computing. Both IOTA and Hashgraph are sort of partial steps from one single blockchain reality toward holochain. But they are still focused on data-centric consensus.
--------
IOTA was created to provide a micro-transactional backbone to Internet of Things economies. Unlike Holochain, IOTA has a fixed currency supply based on the network’s foundational transaction. IOTA also uses a Directed Acyclic Graph (DAG) requiring a high degree of centralization to increase throughput beyond a 2,500 transaction/second limit. Moreover, its ability to scale has been hampered by scarcity of nodes on the network. In IOTA there are no transaction fees because agents do validation work to generate transactions, thus performing the function normally performed by mining blocks. But, in the absence of incentives for running full nodes, the IOT devices and wallets that make up the majority of the network are too light to enable competitive scalability without requiring specialized hardware for IOT devices. More recently, IOTA has been scrutinized for its centralized, closed-source “Coordinator” node run by the IOTA Foundation. 

## Substratum

| [Website](https://substratum.net/)  |
|:-|
| [**White paper**](https://substratum.net/wp-content/uploads/2017/12/Substratum-Whitepaper-English.pdf)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
|  | ✘ |  |  |  | ? |

Substratum operates more like Holo than Holochain, adding incentives to hosts of the decentralized web. Unlike Holochain, Substratum is not using its hosting ecosystem as an engine for the creation of distributed apps. They state no intention to fund, or provide tools for the creation of apps required for the anti-surveillance web (their main promise). Without having designed for flow between two crucial system components, there are unknowns around whether or not there is a market in place for such hosting. 

Substratum is planned to be open sourced once it's working, but no code is visible as of 2018/04/18.

## IPFS

| [Website](https://ipfs.io/)  |
|:-|
| [**White paper**](https://github.com/ipfs/papers/raw/master/ipfs-cap2pfs/ipfs-p2p-file-system.pdf)  |

Holochain is a platform for distributed applications. IPFS is a platform/protocol for distributed file storage. It's possible that some Holochain apps will use IPFS for large, static file storage. The current alpha version of Holochain uses the libp2p library that underlies IPFS. Future versions may not.

## ECSA

| [Website](https://economicspace.agency/)  |
|:-|
| [**White paper**](http://community.ecsa.io/gravity-white-paper)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
| ✔ | ✔ | ✘ | ✔ | ✘ | ✘ |


## Urbit

| [Website](https://urbit.org)  |
|:-|
| [**White paper**](http://media.urbit.org/whitepaper.pdf)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
| ✔ | ✔ | ✘ | ✔ | ✔ | ✔ |

## Radix

| [Website](https://www.radixdlt.com/)  |
|:-|
| [**White paper**](https://www.radixdlt.com/#white-papers)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
| ✔ | ✔ | ✔ | ✔ | ✔ | ✘ |

As of 2018/03/28, Radix has not yet released its Economics White Paper that describes the incentives for running a full-node (comparable to the Holo ecosystem). Like Holochain, Radix has no native currency, but plans to have a low-volatility currency. Unlike Holochain, Radix is still a consensus mechanism.

## Byteball

| [Website](https://byteball.org/)  |
|:-|
| [**White paper**](https://byteball.org/Byteball.pdf)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
| ✘ | ✘ | ✔ | ✔ | ✔ | ✘ |

Byteball’s killer feature is conditional payments, whereby conditions are set for how the payee receives the money. If the condition is not met, the payor gets money back. These design constraints set the conditions for use cases like sports gambling, financial speculation, and buying/selling insurance based on negative events. Its native currency, Bytes are accordingly minted and distributed out of thin air, with the team distributing 99% of all bytes for free in cashback programs, referral rewards, and mass spamming. While Holochain has no native currency, the Holo fuel that pays for web hosting of Holochain applications has an elastic supply/demand formulate based on overall network capacity.

## DAT Project

| [Website](https://datproject.org/)  |
|:-|
| [**White paper**](https://github.com/datproject/docs/blob/master/papers/dat-paper.pdf)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
|  |  |  |  |  |  |

## DADI

| [Website](https://dadi.cloud/)  |
|:-|
| [**White paper**](https://docs.google.com/document/d/1hd1ZDb0NlJwJKbRGi8TGvadqgQZVngTxoOr397ERX24)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
|  |  |  |  |  |  |

Holo’s hosting ecosystem leverages the value of the network to make usage of Holochain apps free to end users, so that more and more people can access such applications and to incent development (since earning fuel means that one has earned web hosting). DADI’s model requires that one buy or earn tokens to use the decentralized web, recreating the high barriers of entry that Holochain circumvents and leaving no incentives for development within the network. The savings DADI touts for web companies seeking hosting on AWS, for example, are offloaded to consumers, making DADI an unlikely candidate for the many popular web services currently offered for free (e.g. Facebook, Twitter, Medium, etc.). 

## Todachain

| [Website](https://www.todachain.com/)  |
|:-|
| [**White paper**](https://github.com/Toufi/Whitepaper/blob/master/TODA_Summary_A_New_Protocol_Approach_V15.pdf)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
| ✔ | ✘ | ✘ | ✘ | ✔ | ✘ |


## Secure Scuttlebutt

| [Website](https://www.scuttlebutt.nz/)  |
|:-|
| [**Docs**](https://ssbc.github.io/docs/) |
| [**Codebase**](https://github.com/ssbc/secure-scuttlebutt)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
| ✔ | ✘ | ✔ |  | ✔ | ✔ |

Secure Scuttlebutt is similar to Holochain in its underlying principles—decentralization and peer-to-peer architecture, cryptographic data integrity, open source—but is specifically for messages and feeds, not generalized for running any app like Holochain.

## Maidsafe

| [Website](https://maidsafe.net/)  |
|:-|
| [**White paper**](https://github.com/maidsafe/Whitepapers/blob/master/Project-Safe.md)?  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
|  |  |  |  |  |  |

Where Maidsafe tethers a coin and mining to information and data stored, which requires speculation on large-scale adoption, Holo’s projections are based on estimates from the crowdsale of hosting boxes. Holo provides dedicated hardware specifically in order to be able to make these projections accurate. MaidSafe's estimate of mining rate over time depend on totally unexplained assumptions about the speed of network adoption and the amount of data stored. On Holo there is no mining and no tokens. This is because Holo fuel is a mutual credit currency that assigns credit limits according to a host's record of hosting services performed. 

Another thing about Maidsafe is that they give no explanation in their whitepaper about their “transaction manager”, which is the underlying fabric that makes their entire system work. Holo’s underlying architecture, Holochain, is quite transparently detailed all over the web. https://holochain.org/ 

The tip that Maidsafe does give about this transaction manager? The fact that they reward what they call 'farmers' (hosts) in a random fashion, which to me speaks volumes about their architecture. Holochain does not need to reward randomly (a la the mining lottery on blockchain) because it can account for each micro-transaction of hosting provision through its crypto-accounting engine. Maidsafe is completely opaque about their consensus process, whereas Holo need not rely on consensus because mutual credit only requires that transactors authenticate their local transaction by auditing each other's source chain to ensure that they have the credits they're spending.  

SAFE also creates its own web browser and requires that adopters download it. Holo was made to allow mainstream net users to access distributed apps through already existing web browsers. The difference is an intent to popularize distributed applications versus the intent to have people create a bunch of new web pages. It’s just a different emphasis. It comes down to whether you’re more interested in encryption and anonymity in all circumstances, which SAFE optimizes, or community engines that fit membranes and identity depending on the purpose of the application. I think of it as the difference between furthering the individual-user perspective of the current net or developing a sense of applications-as-communities, more in the spirit of platform co-ops (many of which rely on user identity).  

Another difference? They take a ton of VC, and so they don't need a crowdsale (even though, to competitively mine Safecoin, one would likely need to buy a ton of hardware from some other provider). 

Yet another? Men to women ratio of team members: Maidsafe 20:3, Holo 20:11.

## Blockstack

| [Website](https://blockstack.org)  |
|:-|
| [**White paper**](https://blockstack.org/whitepaper.pdf)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
| ✔ | ✔ | ✔ | ✔ | ? | ✔ |

## EOS

| [Website](http://eos.io/)  |
|:-|
| [**White paper**](https://github.com/EOSIO/Documentation/blob/master/TechnicalWhitePaper.md)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
| ✔ |  |  |  | ? | ✔ |

EOS is not really a potential competitor to Holo/Holochain, as it seems to be just another, faster blockchain. That doesn't even make it "good enough," I think. Holochain is fundamentally different, and capable of far more interesting things. Regardless of how well EOS does at gaining adoption (i.e., speculative investment) in the short-term, it seems unlikely to me to keep up with something like Holochain in the long-run. Then again, perhaps a super-blockchain project will have great importance in the future, but it will probably serve a different function than something like Holochain, so they may not be competing for the same market slice.

EOS uses Blockchain and a global ledger that will always have scaling issues. 

Native currency.

## ZeroNet

| [Website](https://zeronet.io/)  |
|:-|
| [**Docs**](http://zeronet.readthedocs.io)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
|  |  |  |  |  |  |

# Skycoin

| [Website](https://www.skycoin.net/)  |
|:-|
| [**Whitepaper**](https://www.skycoin.net/downloads/#whitepapers)  |

| Decentralized architecture | dApp environment | Secure hash chains | Application-level data integrity | Scaling | Open source |
|:---:|:---:|:---:|:---:|:---:|:---:|
|  |  |  |  |  |  |

***

Check out the [Holochain Compare & Contrast spreadsheet](https://docs.google.com/spreadsheets/d/1qusS2BohF0_W-vrNrIx7K1_faq8KDFoq_pCnMiSbSTc) to see and contribute to more comparisons.

Here is a recent graphic chart with some different metrics by @emalinus and help from @silversundragon, @giancarlo, and the crew at [Holochain Public Mattermost Compare/Contrast Channel](https://chat.holochain.net/appsup/channels/compare-contrast): 
![Comparison chart](https://i.imgur.com/ZRi1JgJ.png)