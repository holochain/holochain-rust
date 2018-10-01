# Holochain DHT Design Doc

TODO: this page is out of date...

## App Exposed Functions:

### Put(hash)
[ There is no data in this call, because hash is a content hash of the data that will be fetched later by the receiving node when it does validation]

### PutMeta(hash,type,data)

### Validate(sourceNode, hash)
[ Returns source chain data required for validation functions. It could be that the whole source chain is required, or smaller subset. ]

### Get(hash)

### GetMeta(hash,type,range) 
[ Also functions as list returning multiple elements ? ]

## Network/Overlay Functions
Address overlay system which translates between internal node identifiers and cached network addresses.

### Gossip(nodeID,lastKnownIndex)

## Peer to Peer Functions:

### Trackers?
[ Idea: As every new source chain generates their new chain, they also do a PutMeta(HolochainID,"type=Agent",SourceChainID) joins the Holoc is automatically to register themselves as a new Agent in the holochain. Then this is data that gets synchronized to everyone who holds the Holochain data entry. ]

### GetNodes()

### ValidateNode(node hash)

### GetRoles()
*Are roles a Holochain level function? Or application? From a Holochain perspective are all peers equal?*

### GetNodesByRole(role hash)

### NodeRoles(node hash)

