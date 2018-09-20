# Glossary

<dl>
   <dt>Absolutism</dt>
   <dd><p>A view which says there is ONE absolute frame of reference or view of reality. For example, global ledger consensus systems assert the state of the ledger is the one authoritative absolute truth of the system.</p></dd>
   <dt>Agent</dt>
   <dd><p>An entity which operates with agency -- meaning a human or a program, which makes
                    choices outside of whatever code is written into holochain.</p></dd>
   <dt>Application Engine</dt>
   <dd><p>(a.k.a. Application Space) (See Nucleus & Ribosome)</p></dd>
   <dt>Author</dt>
   <dd><p>The agent who originally created the information on their source chain / author chain.</p></dd>
   <dt>Base</dt>
   <dd><p>Provide a hash for the primary resource that you want to define a relation to
Tag: Provide a string that defines the relationship, and which defines how it should be looked up in the future
Link: Provide a hash for the secondary resource that you want to associate with the Base
 It has the side effect of adding the Link+Tag to the base hash on the DHT for each of the links in the list. Thus when you do a getLinks(base,tag) it will return all the links of that tag on that base. This is how we do relations in a sparse hash space, allowing us to convert a hash data store into a graph. Related: the 'commit' method of the holochain API.</p></dd>
   <dt>Blacklist</dt>
   <dd><p>A list of nodes who get blocked for attempting to propagate invalid information.</p></dd>
   <dt>Blockchain</dt>
   <dd><p>An architecture used by Bitcoin, Ethereum and many cryptocurrencies to decentralize maintenance of a global ledger for token accounting.</p></dd>
   <dt>CALM</dt>
   <dd><p>Consistency as Logical Monotonicity. (see Monotonicity)</p></dd>
   <dt>CAP Theorem</dt>
   <dd><p>Consistency-Availability-PartitionTolerance. Often depicted as a triangle, where you can only achieve two of three options. Unfortunately in the real-world, you always have to include 'P' because networks are not 100% reliable. So the question often comes down to are you optimzing for data Consistency or data Availablily (making sure new data is not visible to anyone until its visible to everyone). Holochains are eventually consistent, but our notion of Consistency is not that there is one set of absolutely true data, but rather you know exactly what was asserted by whom and when.</p></dd>
   <dt>Ceptr</dt>
   <dd><p>Short for Receptor. The distributed application and communications framework that Holochains are one small part of. See Ceptr.org for more information.</p></dd>
   <dt>Chain Entry</dt>
   <dd><p>A new transaction added to your source chain.</p></dd>
   <dt>Consensus</dt>
   <dd><p>In the context of decentralized systems, it is the agreement among the nodes about the state of the system.</p></dd>
   <dt>Countersigning</dt>
   <dd><p>When all the agents involved with a transaction sign that transaction to each other's source chains.</p></dd>
   <dt>Cryptographic Proof</dt>
   <dd><p>Used to determine that data is untampered: Hashes are equal and Signatures match.</p></dd>
   <dt>DHT</dt>
   <dd><p>Distributed Hash Table: A structure that enables data to be shared across many machines: easily retrieved, untampered proof by hash.</p></dd>
   <dt>DNA</dt>
   <dd><p>In the context of Holochains, this is the instructions (app code and data schemas) that are valid to operate within that chain. In biology, it is the instructions coded into the nucleus of all the cells of an organism (DeoxyriboNucleic Acid).</p></dd>
   <dt>DPKI</dt>
   <dd><p>Distributed Public Key Infrastructure - A decentralized framework for matching public keys to addresses/users, which often supports other identity data management.</p></dd>
   <dt>Data Positivism</dt>
   <dd><p>The notion that data has "truth" in and of itself. Holochains are Data Relativistic, recognizing no independent truth, only who said what and when.</p></dd>
   <dt>Decentralized</dt>
   <dd><p>The elmination of the need for a central server or authority to maintain system integrity.</p></dd>
   <dt>Distributed</dt>
   <dd><p>A system whose integrity is maintained between ALL nodes of the network (see P2P).</p></dd>
   <dt>End-to-End Encryption</dt>
   <dd><p>The cryptographic keys used to encrypt and decrypt are stored only at the end points.</p></dd>
   <dt>Entry</dt>
   <dd><p>A record in a source chain recorded by the agent who controls that chain.</p></dd>
   <dt>Eventual Consistency</dt>
   <dd><p>A term in distributed computing which refers to the fact that data may be temporarily out of sync in parts of the network, but will eventually become consistent.</p></dd>
   <dt>Flagged</dt>
   <dd><p>Data that has been shared to the DHT, but faills to validate according to the shared rules gets flagged as invalid. Nodes who keep producing invalid data may also get flagged, and then blacklisted.</p></dd>
   <dt>Full Peer</dt>
   <dd><p>A full holochain peer is a machine which is running both as a Source Chain for creating new data and a DHT Node for synchronizing shared data.</p></dd>
   <dt>Genotype</dt>
   <dd><p>In this context, two holochains may have identical genotypes (same code/DNA) but will be used by different groups to be expressed in different ways, thus they are in fact separate and unique. What is different is the group of people and/or what they are communicating to each other. (See phenotype)</p></dd>
   <dt>Global Ledger</dt>
   <dd><p>A centralized and chronological record of all transactions in a particular system.</p></dd>
   <dt>Gossip</dt>
   <dd><p>Gossip refers to the communications that nodes in the DHT do to keep each other up to date with the data they're supposed to be collectively holding, and managing.</p></dd>
   <dt>Hash</dt>
   <dd><p>A mathematical algorithm that compactly maps data in a one-way function that is infeasible to invert. The workhorse of modern cryptography.</p></dd>
   <dt>HashChain</dt>
   <dd><p>The successive use of cryptographic hash functions to a piece of data that allows many single-use keys to be created from a single key.</p></dd>
   <dt>Holochain</dt>
   <dd><p>A holochain is a validating distributed hash table (DHT) where every node enforces validation rules on data against the signed chains where the data originated.</p><p>In other words, a holochain functions very much&nbsp;<strong>like a blockchain without bottlenecks</strong>&nbsp;when it comes to enforcing validation rules, but is designed to be fully distributed through sharding so each node only needs to hold a portion of the data instead of a full copy of a global ledger. This makes it feasible to run blockchain-like applications on devices as lightweight as mobile phones</p></dd>
   <dt>HolochainID</dt>
   <dd><p>The Hash by which a holochain is known . This is the hash of the holochain's DNA (application code and data schemas) combined with unique identifying information about the group using this DNA.</p></dd>
   <dt>Immune System</dt>
   <dd><p>Holochains are designed for eventual consistency and for detecting attempts to compromise data consistency by bad actors on the network. We refer to the mechanisms encoded in a holochain to protect itself as the holochain's immune system. For example: blacklisting someone who keeps posting invalid data, or counterfeits data as if it was created by others.</p></dd>
   <dt>Links</dt>
   <dd><p>A links entry that you commit to your chain is a list of links. There are three parts to each link: </p></dd>
   <dt>Merkle Proof</dt>
   <dd><p>A mechanism that extends the ability to authenticate a small amount of data to allowing the authentication of large databases. E.G. A user of a Merkle tree with a publicly known and trusted root can ask for a Merkle Proof to verify any value in the tree is correct.</p></dd>
   <dt>Merkle Tree</dt>
   <dd><p>A hash tree that allows efficient and secure verification of the contents of large data structures (see hash tree).</p></dd>
   <dt>MetaData</dt>
   <dd><p>(in the DHT) Our DHT allows people to store data (which gets hashed and then stored at the address that is the hash of the data), as well as store metadata about that the data at that hash, such as who has verified it, or what data it links to.</p></dd>
   <dt>Monotonic</dt>
   <dd><p>In the case of holochain, this refers to the idea that data elements only increase, never decrease. In distributed systems, it can be very hard to synchronize the removal of data, so we keep a record of the existence of the data, and mark it deleted (or expired, or flagged as invalid). But if you remove the data, there's nothing to attach the synchronization markers to.</p></dd>
   <dt>Multi-party Transaction</dt>
   <dd><p>A transaction which involves multiple different agents, and must be countersigned by all parties to each of their source chains before propagation on the shared DHT.</p></dd>
   <dt>Neighborhood</dt>
   <dd><p>As a DHT grows larger with more and more members, it is segmented or sharded into neighborhoods which manage data together. Nodes in a neighborhood "gossip" to update each other, and also track participation information to make sure their neighbors haven't gone rogue. (See Immune System)</p></dd>
   <dt>Node</dt>
   <dd><p>A node is a machine participating in the DHT peer-to-peer communications involved in sharing and validating data.</p></dd>
   <dt>NodeID</dt>
   <dd><p>The address of a node in the DHT.</p></dd>
   <dt>Nucleus</dt>
   <dd><p>The Application Container for executing the instructions in the DNA of the Holochain application. The DNA is split in "Zomes" (i.e. chromosome) that may be written in different programming languages. The Nucleus contains language specific "Ribosomes" that provide a virtual machine for executing the DNA code.</p></dd>
   <dt>P2P</dt>
   <dd><p>Peer-to-Peer: An approach to distributed system development where every peer is an equal to other peers and they coordinate in that manner.</p></dd>
   <dt>P3</dt>
   <dd><p>Protocol for Pluggable Protocols - Ceptr's self-describing protocols stack which enables interoperability between holochains.</p></dd>
   <dt>Phenotype</dt>
   <dd><p>In this context, a phenotoype is the way in which two holochains with identical code (genotype) express themselves in different ways thus making them separate and unique. What is different is the group of people and/or what they are communicating to each other. (See genotype)</p></dd>
   <dt>Private Key</dt>
   <dd><p>A key known only to the recipient in a cryptographic system that is used to decrypt a message (see public key).</p></dd>
   <dt>Provenance</dt>
   <dd><p>The official record of origin of data.</p></dd>
   <dt>Public Key</dt>
   <dd><p>A key known to all nodes in a cryptographic system that is used to encrypt a message.</p></dd>
   <dt>Ribosome</dt>
   <dd><p>A virtual machine that runs DNA code in a particular programming language. Currently implemented are JavaScript and Lisp Ribosomes.</p></dd>
   <dt>SNARK</dt>
   <dd><p>Succinct Non-interactive ARgument of Knowledge - A form of Zero Knowledge Proof that can be used for showing validation of a particular process.</p></dd>
   <dt>Schema</dt>
   <dd><p>A definition used to define what data can be used in a context, as well as some parameters for validating that data. (is it required, in a specific range, etc.)</p></dd>
   <dt>Semantic Tree</dt>
   <dd><p>A native data structure of Ceptr. Trees are used to show the structure of data and each node in the tree has a semantic marker referencing its definition and methods.</p></dd>
   <dt>Semtrex</dt>
   <dd><p>Semantic Tree Regular Expressions: A universal parsing system for matching against semantic trees.</p></dd>
   <dt>Shard</dt>
   <dd><p>A large DHT gets segmented or sharded into neighborhoods which manage data together. This allows the data to be shared without everybody needing to possess a full copy of ALL the data (as in a global ledger).</p></dd>
   <dt>Shared Store</dt>
   <dd><p>The wholeness of holochains come from combining local signed source chains with a shared data store via DHT.</p></dd>
   <dt>Signature</dt>
   <dd><p>A cryptographic signature is usually created by creating a cryptographic hash of some data and encrypting that hash with your private key. This proves it was you who signed it (or at least someone who had your keys, and that the data being signed hasn't been altered because it resolves to the expected hash).</p></dd>
   <dt>Source</dt>
   <dd><p>Typically refers to the agent or person that authored data or sent a message.</p></dd>
   <dt>Source Chain</dt>
   <dd><p>(a.k.a Authoring Chain) This is the local signed hash chain that you commit new data to before sharing it to the validating DHT.</p></dd>
   <dt>SourceID</dt>
   <dd><p>The identity of the source of a particular message, or piece of data or metadata.</p></dd>
   <dt>TimeStamp</dt>
   <dd><p>Holochain activities are recorded with the time and date something happened according to the time on their machine. Holochains do not have a guaranteed global time, but may refuse to synchronize transactions with nodes whose clocks are too far out of sync.</p></dd>
   <dt>User</dt>
   <dd><p>In the context of data attribution it refers to the Author or Agent who created the data. In the context of an Application or UI/UX, it refers to the person using the app (seeing the data).</p></dd>
   <dt>Validating DHT</dt>
   <dd><p>A DHT where every node executes consistent validation rules on data before propagating that data.</p></dd>
   <dt>Validation</dt>
   <dd><p>Confirming that data is valid according to the shared rules of a holochain. This should happen before data is committed to your source chain, and must also happen as data is propagated across a DHT.</p></dd>
   <dt>Validation Rules</dt>
   <dd><p>The rules which enforce valid data that can be committed to source chains as well as data that can propagate via the DHT.</p></dd>
   <dt>Zero Knowledge Proof</dt>
   <dd><p>A method in which one agent can prove to another agent that something is true, without having to expose additional information except that it is true.</p></dd>
   <dt>Zome</dt>
   <dd><p>(as in Chromosome) Each Nucleus can contain multiple Zomes that may have been mixed in when the Holochain application was written. All data elements committed to source chain or propagated on the DHT, comply with the data schema and validation rules of a particular Zome.</dd>
</dl>
