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
Distributed systems rely more heavily on cryptographic patterns and techniques than centralized systems. 

### Hashes

### Signatures 

### Encryption

## Entry types 
You can define particular entry types as public, private, or encrypted. Encryption is an in-built option for Holochain apps, meaning that you can automatically encrypt entry types by default.
This is because Holochain's use of DHTs means that simply running the application allows user-participants access to all entries on the network.

### Public entries
Public entries are published to the app's DHT with an appropriate redundancy factor. 

### Encrypted entries
Like public entries, encrypted entries are published to the app's DHT with an appropriate redundancy factor.

### Private entries
Unlike public and encrypted entries, private entries are only on the source chain (device) of the publisher and are not shared out to the DHT. 
Of course, one could always encrypt a private entry, too. 

## Security - best practices
A great way to begin offsetting the governance crises now typical of distributed systems (i.e. DAO hack) is to think in terms of protecting and enabling the community of user-participants in addition to cryptography.    

In essence, one must consider how to prevent undesired access to the DHT. If membranes are not properly built in the dApps's DNA, having access to the source code also means having access to the entire network's entries via the DHT. Developers must treat the code, or at least the DNA taken as a whole, as if it's a key to the data. Note, too, that one can easily fork a Holochain dApp without disrupting the its activity, making it possible to retain the benefits of open-source code without some of the risks.

### Membranes

Therefore, security efforts begin with the specification of membranes, lest the code itself become a target. Though holochains rely on the cryptography above to create trust in data's provenance and immutability, trust is a distinctly human affair at the level of creating membranes. Different applications will require different levels of security, and Holochain is uniquely suited to accommodate a high degree of flexibility. DNA could define a closed list of participants, Proof of Service, or social triangulation requirements, for example.  
 
## Scenarios to consider
1. p2p platforms
2. supply chains and open value networks
3. social networks 
4. collaboration apps
