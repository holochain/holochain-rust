# Planning a dApp

## Introduction
Generally speaking, you need to know the following in order to build a holochain dApp:

   how to install holochain
   
   how to use the command line tools
   
   how to configure your application with a "DNA" file
   
   how to write your application code in JavaScript or Lisp
   
<b>how to think through building a distributed application</b> 

   how to build a user interface for your app
   
   how to test your application code

This article will help you plan a dApp by providing practical considerations about the specifics of distributed applications in general, and holochain dApps in particular. It has been remarked that holochain dApps require us to make a mental shift, first from applications whose data is centrally organized, and also from blockchain-based, data-centric dApps.

Here we will provide a basic overview of concepts from cryptography that are central to holochains. 
Then, we will consider the consequences of holochain's cryptographic architecture for data permissioning, access, and security. 

## What is a dApp? 
A dApp is a distributed application. This means that the data associated with the application is stored by each user rather than in a central database.  

## Basic expectations for dApps
Because app data storage is distributed amongst user-participants, one must expect that data encryption and permissions are important for protecting privacy in accordance with the jurisdictions in which the app is operating.

Remember that, as user-participants leave the application, they take their data with them. They also retain copies of other data that they held to support the DHT.  

One must also re-think the dApp's business model such that it does not rely on a central authority's ability to whitelist access to a given resource. 

## Cryptography in holochain dApps
Distributed systems rely more heavily on cryptography than centralized systems. 

### Hashes

### Signatures 

### Encryption

## Entry types 
You can define particular entry types as public, private, or encrypted. Encryption is an in-built option for holochain apps, meaning that you can automatically encrypt entry types by default.
This is because holochain's use of DHTs means that simply running the application allows user-participants access to all entries on the network.

### Public entries
Public entries are published to the app's DHT with an appropriate redundancy factor. 

### Encrypted entries
Like public entries, encrypted entries are published to the app's DHT with an appropriate redundancy factor.

### Private entries
Unlike public and encrypted entries, private entries are only on the source chain (device) of the publisher and are not shared out to the DHT. 
Of course, one could always encrypt a private entry, too. 
Consider the use cases of a private blockchain...

## Security - best practices
Different applications will require different levels of security. Holochain is uniquely suited to accommodate a high degree of flexibility, however there are generic best practices based on its native architecture. In essence, one must consider how to prevent undesired access to the DHT. After all, if someone has access to the source code, the app's DNA, they also have access to the DHT. This means that everyone who uses the app potentially has access to all the network's entries. Developers must treat the code as if it's a key to the data, effectively shifting their security efforts to the code itself rather than where sensitive data is stored. Though holochains rely on the cryptography above to create trust in data's provenance and immutability, trust is a distinctly human affair at the level of access to the code. A great way to begin offsetting the governance crises now typical of distributed systems (i.e. DAO hack) is to think in terms of protecting and enabling the community of user-participants in addition to cryptography.    

One can easily fork a holochain dApp without disrupting the app's activity, making it possible to retain the benefits of open-source code.   
 
## Scenarios to consider
1. p2p platforms
2. supply chains and open value networks
3. social networks
4. collaboration apps 
