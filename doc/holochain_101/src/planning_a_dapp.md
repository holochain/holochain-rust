# Planning a dApp

## What is a dApp? 
A dApp is a distributed application. This means that the data associated with the application is stored by each user rather than in a central database.  

## Basic expectations for dApps
Generally speaking, you need to know the following in order to build a Holochain dApp:

   how to install Holochain
   
   how to use the command line tools
   
   how to configure your application with a "DNA" file
   
   how to write your application code in a language that compiles to WebAssembly 
   
   **how to think through building a distributed application** 

   how to build a user interface for your app
   
   how to test your application code

This article will help you plan a dApp by providing practical considerations about the specifics of distributed applications in general, and Holochain dApps in particular. It has been remarked that holochain dApps require us to make a mental shift, first from applications whose data is centrally organized, and also from blockchain-based, data-centric dApps.

Here we will provide a basic overview of concepts from cryptography that are central to holochains. 
Then, we will consider the consequences of Holochain's cryptographic architecture for data permissioning, access, and security. 
Because app data storage is distributed amongst user-participants, one must expect that data encryption and permissions are important for protecting privacy in accordance with the jurisdictions in which the app is operating.

Remember that, as user-participants leave the application, they take their data with them. They also retain copies of other data that they held to support the DHT.  

One must also re-think the dApp's business model such that it does not rely on a central authority's ability to whitelist access to a given resource. 

## Cryptography in Holochain dApps
Distributed systems rely more heavily on cryptographic patterns and techniques than centralized systems. The basic concepts below explain how data integrity, ownership, and security are achieved natively with holochain's architecture. They are time-worn, relatively intuitive ideas that are critical for planning a holochain dApp. 

### Hashes
Hashes ensure the reliability of information by representing any given piece of data with a unique, consistent string of random looking characters. This makes changes to data visible because one can see that a hash has changed without needing to inspect the data itself.  

However, it is impossible to get the original data from a hash -- its purpose is to prove that the data to which it corresponds has not been altered. The same data consistently gives the same hash, and different data always give a completely different hash. 

These features imply that one can use small, portable hashes to verify data. One could also use a database containing data and their hashes as a table of contents, indexing (though not reading) data associated with a given hash. 

In the context of Holochain hashes are frequently used to look up content, both in our internal implementations as well as on the DHT.  Therefore we frequently refer to the hash of some item (i.e. an entry on the chain) as its *Address*.
### Signatures 
Signatures provide an additional type of data verification, answering the question "who created this data?" Signatures look like hashes. They are unique, reliable, and like hashes, cannot be used to retrieve the data to which they correspond. Signatures also come with a pair of keys. One is public, and the other private. 

The private key designates a unique author (or device), and the public key lets anyone verify a signature made by one specific private key. This key infrastructure addresses the problem of single points of failure associated with centralized systems by making authors responsible for securing their unique private key.   

### Encryption
What if one needs to restrict access in addition to verifying data? Two types of encryption are possible. _Symmetric_ encryption has one key for reading and writing data. _Asymmetric_ encryption has two keys, where one creates messages and the other reads them.

Encryption is a two way process, so the right key enables one to decrypt an encrypted message. With this added benefit come the drawbacks of the size of encrypted messages (at least as large as the original data) and broken encryption stripping the author of control of the original data. 

## Data access paradigms
The following are five data access paradigms. Note that in real-world scenarios it is common to mix these options by combining separate dApps. 
In instances when many separate dApps are needed to share data, Holochain supports bridging between dApps.
Bridges between two networks with different data privacy models specify who can use the bridge, what data crosses the bridge, and tasks that might run in response to the bridge (e.g. notifications)

The default model for Holochain data is public data shared on a public network, and every Holochain dApp has its own network and data, and creates networks for user-participants as soon as they join a dApp. 
The dApp code sets sharing and verification rules. 

### Public, shared data on a public network

Public data works like Bittorrent: 

   - anybody can join a network
   - anybody can request any data they want from the network
   - any data is available as long as at least one person is sharing it
   - if some data is not shared by enough people, a new random person on the network must share it
   - there is no "local only" data  

As stated above, an additional requirement for Holochain dApps is that new data must have a digital signature. 

### Public, shared data on a private network
The functionality is the same as a public network, but private networks use cryptography for access control to the network itself.

Each device on the network must open a P2P connection to another device before it can send any data. 
The devices that are already on the private network send a challenge to any new device before sending any more data. 
The new device must sign the challenge with the network private key. 
The network public key is set in the dApp configuration, available to Holochain. 
Holochain can then refuse any connection with a bad challenge signature.
Data within the network is public and shared. Every device on the network has “logged in” with a signed challenge, so has full access.

### Encrypted data on a public or private network
Encryption relies on dApp developers encrypting and decrypting data within the dApp software.

Holochain exposes a set of industry standard encryption algorithms (e.g. AES) to each dApp that cover both symmetric and asymmetric encryption options, in addition to hashing and signing tools.  

This option is very flexible for dApp developers but security becomes more subtle.
Much like the private network, any one user-participant losing a key can become a serious security problem.

Note that encryption can pose problems for Holochain's native validation method. 

### Private, local only data
Any data in a dApp can be validated and stored by its author without being shared to the network. Private, local data can provide a useful database for portable user preferences and notes and avoids the complexity of encryption and key-based security. 

Private data is hashed in the same way as public data, and the hash is public. Accordingly, one could tell that private data exists without being able to access it or take advantage of this with dApps that feature the eventual reveal of previously authored, private data -- think a distributed guessing game, like "rock, paper, scissors" or a digital classroom that operates with signatures disconnected from real-world identity and uses this method to prevent cheating. 

### Hybrid model with servers
Holochain supports many different environments and interfaces so that Holochain is easy to integrate with existing infrastructure. Any connected system with an API can push data through a dApp, as when one's phone sends a summary of private calendar data to a Holochain dApp. Any data in a dApp immediately becomes transparent, auditable and portable.

The version of Holochain in active development covers the following integrations:

* Command line tools
* Web server
* Android
* Qt/QML
* Unity 3D games engine

## Security - best practices
A great way to begin offsetting the governance crises now typical of distributed systems (i.e. DAO hack) is to think in terms of protecting and enabling the community of user-participants in addition to cryptography.    

In essence, one must consider how to prevent undesired access to the DHT. If membranes are not properly built in the dApps' DNA, having access to the source code also means having access to the entire network's entries via the DHT. Developers must treat the code, or at least the DNA taken as a whole, as if it's a key to the data. Note, too, that one can easily fork a Holochain dApp without disrupting its activity, making it possible to retain the benefits of open-source code without some of the risks.

### Membranes

Security efforts begin with the specification of membranes, lest the code itself become a target. Though holochains rely on the cryptography above to create trust in data's provenance and immutability, trust is a distinctly human affair at the level of creating membranes. Different applications will require different levels of security, and Holochain is uniquely suited to accommodate a high degree of flexibility. DNA could define a closed list of participants, Proof of Service, or social triangulation requirements, for example.  

Sybil attacks are attacks launched through the creation of many nodes explicitly for the purpose of the attack. These nodes are identifiable by having no history. Blockchains prevent Sybil Attacks with Proof of Work. PoW is an implied membrane since it constrains who can verify blocks. In that case, the clearing node must have done "work" to maintain the network. Holochain requires membranes to identify and filter out Sybil nodes so that an attacker cannot use them to overrun the DHT.

### Immune system
Holochain relies on random users across the network validating every piece of data. This keeps the verification rules reliable and unbiased. This is called an "immune system" for validating content. 

Note that when using encrypted data, it is not possible to verify contents without the appropriate key. Encryption is a choice that should be made carefully, as it undermines Holochain's native immune system. 
 
## Scenarios to consider
1. p2p platforms
2. supply chains and open value networks
3. social networks 
4. collaboration apps
