# Glossary

<dl>
   <dt>Absolutism</dt>
   <dd><p>The idea that there is always only ONE absolute shared frame of reference or view of reality. For example, global ledger consensus systems treat the shared ledger data as the absolute truth of the system at all times.</p></dd>
   <dt>Agent</dt>
   <dd><p>An entity operating with agency. A human or software agent makes decisions outside of the holochain code.</p></dd>
   <dt>Application Engine</dt>
   <dd><p>(a.k.a. Application Space) (See Nucleus & Ribosome)</p></dd>
   <dt>Author</dt>
   <dd><p>The agent who originally created the information on their source chain / author chain.</p></dd>
   <dt>Blacklist</dt>
   <dd><p>A list of nodes that are blocked for attempting to propagate invalid information.</p></dd>
   <dt>Blockchain</dt>
   <dd><p>An architecture used by Bitcoin, Ethereum and many cryptocurrencies to decentralize maintenance of a global ledger of data.</p></dd>
   <dt>CALM</dt>
   <dd><p>Consistency As Logical Monotonicity. (see Monotonicity)</p></dd>
   <dt>CAP Theorem</dt>
   <dd><p>A theorem describing the limitations of sharing stateful data across a network. CAP stands for Consistency, Availability, Partition Tolerance. Consistency means that data requested from two parts of the network has the same value at the same time. Availability means that data can be requested and retrieved immediately from any part of the network. Partition Tolerance means that gaps in network connectivity do not break Consistency or Availability. Consistency and Availability are both possible while the network has no Partitions but real world networks are frequently partitioned. (see Eventual Consistency and PACELC theorem).</p></dd>
   <dt>Ceptr</dt>
   <dd><p>Short for Receptor. The distributed application and communications framework that Holochains are one small part of. See [Ceptr.org](https://ceptr.org/) for more information.</p></dd>
   <dt>Chain Entry</dt>
   <dd><p>A unit of data added to an agent's source chain.</p></dd>
   <dt>Consensus</dt>
   <dd><p>In the context of decentralized systems, it is the agreement among the nodes about the state of the system.</p></dd>
   <dt>Countersigning</dt>
   <dd><p>When all agents involved in a transaction sign that transaction and store it as an entry in each other's source chains.</p></dd>
   <dt>Cryptographic Proof</dt>
   <dd><p>A mathematical proof that some data has not been tampered with (hash) or that it was created by a specific author (signature). Usually multiple cryptographic proofs are combined to create a functioning system.</p></dd>
   <dt>Distributed Hash Table (DHT)</dt>
   <dd><p>A structure that enables data to be shared across many machines and easily retrieved by its cryptographic hash. The hash facilitates both efficient lookup and verification that the content has not been tampered. (see Cryptographic Proof)</p></dd>
   <dt>DNA</dt>
   <dd><p>In Holochain: the instructions (app code and data schemas) that are valid for a given network/DHT.</p>
      <p>In biology: the instructions coded into the nucleus of every cell in an organism (DeoxyriboNucleic Acid).</p></dd>
   <dt>Distributed Public Key Infrastructure (DPKI)</dt>
   <dd><p>A decentralized system to lookup the address/user for a given public key. Supports translating low level cryptographic proofs (signatures) into trustworthy real-world identity data. (see Cryptographic Proof)</p></dd>
   <dt>Data Positivism</dt>
   <dd><p>The idea that data represents absolute "truth" in and of itself. Holochain does not subscribe to this idea. (see Data Relativism).</p></dd>
   <dt>Data Relativism</dt>
   <dd><p>The idea that data only has meaning in an interpreted context. Holochain is Data Relativistic, the interpreted data is what each agent is aware of what data other agents broadcast and when. (see Data Positivism).</p></dd>
   <dt>Decentralized</dt>
   <dd><p>Systems that maintain integrity without supervision from priviledged servers or authorities.</p></dd>
   <dt>Distributed</dt>
   <dd><p>Systems requiring all participants to share the responsibility of maintaining network integrity. (see P2P).</p></dd>
   <dt>End-to-End Encryption</dt>
   <dd><p>The cryptographic keys used to encrypt and decrypt are stored only at the end points of a communication. Data is not and cannot be decrypted at any point between the sender and receiver.</p></dd>
   <dt>Entry</dt>
   <dd><p>A record in a source chain recorded by the agent who controls that chain. Public entries are broadcast to the DHT. (see DHT).</p></dd>
   <dt>Eventual Consistency</dt>
   <dd><p>Distributed computing term. New data broadcast across an eventually consistent system may be temporarily inconsistent but is guaranteed to eventually become consistent. Holochain is eventually consistent. Consistency in Holochain does not mean global consensus but that each agent knows exactly what was asserted by whom and when. (see Consistency).</p></dd>
   <dt>Flagged</dt>
   <dd><p>Data shared on the DHT that fails to validate. Validation rules are set in the DNA. Nodes that regularly author or share invalid data may be flagged and/or blacklisted.</p></dd>
   <dt>Full Peer</dt>
   <dd><p>A full holochain peer is a machine running both authoring new entries to its local source chain and participating as a DHT Node for synchronizing shared data. (see DHT and Source Chain).</p></dd>
   <dt>Genotype</dt>
   <dd><p>In Holochain: two hApps with identical genotypes (same code/DNA) but used by different groups to be _expressed_ in different ways. The different expression separates the network, participants and data. (See phenotype)</p></dd>
   <dt>Global Ledger</dt>
   <dd><p>A globally shared, immutable and chronological record of all transactions in a particular system.</p></dd>
   <dt>Gossip</dt>
   <dd><p>Communications between nodes in the DHT to collaboratively organise what data each node should be holding, and managing.</p></dd>
   <dt>Hash</dt>
   <dd><p>A mathematical algorithm that compactly maps data in a one-way function that is infeasible to invert. The workhorse of modern cryptography. (see Cryptographic Proof).</p></dd>
   <dt>Hash Chain</dt>
   <dd><p>The successive use of cryptographic hash functions to a piece of data that allows many single-use keys to be created from a single key.</p></dd>
   <dt>Holochain network</dt>
   <dd><p>A holochain network is a validating distributed hash table (DHT). Every node enforces validation rules on DHT data against the signed chains where the data originated.</p>
      <p>A holochain network collectively achieves similar security and data distribution to a blockchain system.</p>
      <p>Holochain networks are designed very differently to blockchains and are more scalable, using cheaper commodity devices to participate in.</p></dd>
   <dt>Holochain ID</dt>
   <dd><p>The hash identifying and allowing access to a holochain network. This is the hash of the holochain's DNA (application code and data schemas) combined with unique identifying information about the group using this DNA.</p></dd>
   <dt>Immune System</dt>
   <dd><p>Holochains are designed for eventual consistency and to detect malicous attempts to compromise data consistency. These security and reliability mechanisms are the holochain network's immune system. For example: blacklisting nodes that regularly post invalid or counterfeit data.</p></dd>
   <dt>Links</dt>
   <dd><p>A links entry committed to a chain as a list of links. Directed relations convert a sparse hash space/storage into a graph. Links are a directed reference between two hash addresses. There are three parts to each link:</p></dd>
   <dt>Link Base</dt>
   <dd><p>The hash of the primary resource the link relationship points from.</p></dd>
   <td>Link Tag</dt>
   <dd><p>A string defining the type of relationship, and how it should be looked up in the future</p></dd>
   <dt>Link Target</td>
   <dd>The hash of the secondary resource the link relationship points to.</p></dd>
   <dt>Merkle Proof</dt>
   <dd><p>Mathematical proof to show that any small part of data is part of a much larger set of data and has not been tampered with. The proof does not require access to the entire large data set to work. The proof relies on a data structure called a Merkle Tree. (see Merkle Tree).</p></dd>
   <dt>Merkle Tree</dt>
   <dd><p>A hash tree that allows efficient and secure verification of the contents of large data structures. Something like a blockchain combined with a binary search. e.g. A user of a Merkle tree with a publicly known and trusted root can ask for a Merkle Proof to verify any value in the tree is correct. (see hash tree and Merkle Proof).</p></dd>
   <dt>Metadata</dt>
   <dd><p>(in the DHT) Our DHT allows people to store data (which gets hashed and then stored at the address that is the hash of the data), as well as store metadata about that the data at that hash, such as who has verified it, or what data it links to.</p></dd>
   <dt>Monotonic</dt>
   <dd><p>The idea that stored data only increases, never decrease. This means all data points are kept then aggregated (e.g. using a `max` function) rather than updated in place. In distributed systems, it is very hard to synchronize the removal of data, so we keep a record of the existence of the data, and mark it retracted (or expired, or flagged as invalid). A retraction is not a deletion of the original data but an addition of new data that asserts the old data is outdated.</p></dd>
   <dt>Multi-party Transaction</dt>
   <dd><p>A transaction which involves multiple different agents, and must be countersigned by all parties to each of their source chains before propagation on the shared DHT.</p></dd>
   <dt>Neighborhood</dt>
   <dd><p>Each holochain network has a configured redundancy. Each entry is copied this many times to different nodes maintaining the DHT. Each entry is sent to the nodes with the hashes most similar to the entry's own hash. This forms a "neighbourhood" for the nodes around that entry. The size of the neighbourhoods is the redundancy of the network. Nodes in a neighborhood "gossip" to each other to track participation in the storage of the entry and to make sure their neighbors haven't gone rogue. If a node drops or cannot produce the entry when requested then it is replaced in the network by the node with the next most similar node ID to the entry hash. (See Immune System)</p></dd>
   <dt>Node</dt>
   <dd><p>A node is a machine participating in the DHT peer-to-peer communications involved in sharing and validating data.</p></dd>
   <dt>Node ID</dt>
   <dd><p>The address of a node in the DHT.</p></dd>
   <dt>Nucleus</dt>
   <dd><p>The Application Container for executing the instructions in the DNA of the Holochain application. The DNA is split in "Zomes" (i.e. chromosome) that may be written in different programming languages. The Nucleus contains language specific "Ribosomes" that provide a virtual machine for executing the DNA code.</p></dd>
   <dt>PACELC theorem</td>
   <dd><p>An extended form of CAP theorem to include latency even when there are no network partitions. Reads like "if Partition then Availability or Consistency else Latency or Consistency". (see CAP theorem).</p></dd>
   <dt>P2P</dt>
   <dd><p>Peer-to-Peer: An approach to distributed system development where every peer is an equal to other peers and they coordinate in that manner. (see Distributed)</p></dd>
   <dt>P3</dt>
   <dd><p>Protocol for Pluggable Protocols - Ceptr's self-describing protocols stack which enables interoperability between holochain networks.</p></dd>
   <dt>Phenotype</dt>
   <dd><p>In this context, a phenotoype is the way in which two holochains with identical code (genotype) express themselves in different ways thus making them separate and unique. What is different is the group of people and/or what they are communicating to each other. (See genotype)</p></dd>
   <dt>Private Key</dt>
   <dd><p>A secret key that can encrypt, decrypt or sign some data. The functionality of a private key is specific to each cryptographic system. The private key is used to perform sensitive operations. In symmetric cryptographic systems there is only one key, the private key (e.g. password for encryption/decryption). In asymmetric cryptographic systems there is a complementary public key to handle the inverse operation (e.g. sign vs. verify). (see public key).</p></dd>
   <dt>Provenance</dt>
   <dd><p>The official record of origin of data.</p></dd>
   <dt>Public Key</dt>
   <dd><p>The companion key to a private key in an asymmetric cryptographic system. The functionality of a public key is specific to each cryptographic system. The public key is used to verify sensitive operations without allowing the key holder to perform sensitive operations (e.g. verifying a signature). (see private key).</p></dd>
   <dt>Ribosome</dt>
   <dd><p>DNA code written in a some programming language to be executed by nodes in the holochain network.</p></dd>
   <dt>SNARK</dt>
   <dd><p>Succinct Non-interactive Argument of Knowledge - A form of Zero Knowledge Proof that can be used for showing validation of a particular process.</p></dd>
   <dt>Schema</dt>
   <dd><p>A definition used to define what data can be used in a context, as well as some parameters for validating that data. (is it required, in a specific range, etc.)</p></dd>
   <dt>Semantic Tree</dt>
   <dd><p>A native data structure of Ceptr. Trees are used to show the structure of data and each node in the tree has a semantic marker referencing its definition and methods.</p></dd>
   <dt>Semtrex</dt>
   <dd><p>Semantic Tree Regular Expressions: A universal parsing system for matching against semantic trees.</p></dd>
   <dt>Shard</dt>
   <dd><p>A subset of a large data set. Holochain networks have local DHT shards composed of all the entries each node is in the neighbourhood of. The size of each holochain network shard is the average data produced by each node multiplied by the network redundancy. Higher redundancy means better availability, lower latency and less risk of data loss but also increased storage and network costs. Infinite redundancy means all nodes hold all data like a full node in a blockchain system. (see neighbourhood)</p></dd>
   <dt>Shared Store</dt>
   <dd><p>The wholeness of holochains come from combining local signed source chains with a shared data store via DHT.</p></dd>
   <dt>Signature</dt>
   <dd><p>A cryptographic signature is usually created by creating a cryptographic hash of some data and encrypting that hash with your private key. This proves it was you who signed it (or at least someone who had your keys, and that the data being signed hasn't been altered because it resolves to the expected hash).</p></dd>
   <dt>Source</dt>
   <dd><p>The agent or person that authored data or sent a message.</p></dd>
   <dt>Source Chain</dt>
   <dd><p>(a.k.a Authoring Chain) This is the local signed hash chain that data is committed to. Public entries are shared to the DHT after they are committed to the local chain. Private entries are not shared to the DHT but are available to the authoring agent in their local chain.</p></dd>
   <dt>Source ID</dt>
   <dd><p>The identity of the source of a particular message, or piece of data or metadata.</p></dd>
   <dt>Timestamp</dt>
   <dd><p>Holochain activities are recorded with the time and date something happened according to the time on their machine. Holochain networks have to guaranteed global time, but may refuse to synchronize transactions with nodes whose clocks are too far out of sync.</p></dd>
   <dt>Trusted timestamp server</dt>
   <dd><p>A centralised server that signs data with a timestamp. Can be used to prove that some data existed at or before some time and has not been tampered with since. The proof relies on trusting the timestamp server to protect their private key and not allow fake times. Some existing trusted timestamp servers are maintained by the same organisations that issue SSL certificates. Blockchain is a distributed trusted timestamping service that tracks time in blocks rather than seconds.</p></dd>
   <dt>User</dt>
   <dd><p>In the context of data attribution it refers to the Author or Agent who created the data. In the context of an Application or UI/UX, it refers to the person using the app (seeing the data).</p></dd>
   <dt>Validating DHT</dt>
   <dd><p>A DHT where every node executes consistent validation rules on data before propagating that data.</p></dd>
   <dt>Validation</dt>
   <dd><p>Confirming that data is valid according to the shared rules of a holochain network. Validation is done before data is committed to honest source chains, and again every time it is shared to an honest DHT participant. Invalid data is not committed locally but is stored if it is broadcast to the network.</p></dd>
   <dt>Validation Rules</dt>
   <dd><p>Rules that define what data can be committed to honest source chains and what data can propagate to the DHT.</p></dd>
   <dt>Zero Knowledge Proof</dt>
   <dd><p>Mathematical proof that allows an agent can prove to another agent that something is true without exposing any additional information.</p></dd>
   <dt>Zome</dt>
   <dd><p>(as in Chromosome) Each nucleus can contain many zomes. Each zome is a logical bundle of software code that defines the validation rules used by the holochain network. (see validation rules).</dd>
</dl>
