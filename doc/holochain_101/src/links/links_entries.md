# Links Entries

A link consists of 4 parts : 
`Base` - the address of the entry on which the links will be stored in the DHT.
`Link Type` - a String that corresponds to a value that we would use to give the link a type
`Tag` - an arbitrary string which can be added to link when it is created. This s accessible to the validation callback and can be used when retrieving links to filter by regex.
`Target` - that address of the entry to be linked to from the base

The process of `linking` in holochain is done through the `link_entries` function in the HDK, this will allow the zome developer to connect different kinds of data. All data that is linked is stored in our `EAV storage`(see holochain-persistance-api for more details on how this works). The `EAV` is the backbone of our storage mechanism when it comes to our linking process and addition and retrival of links is done using it. 

# Validation

`System Validation`
They are two layers of validation when it comes to linking. One layer happens at the system level and this executes every time a link is added and uses validation rules which are defined by the system. The primary validation is to ensure that the base address exists. Since this corresponds to real data on the DHT we have to make sure that the hash we are giving it is correct and corresponds to something.

`Zome Validation`
Links also allow for the Zome Developer to define what rules to be checked before a link will be added to the DHT. The Zome Developer can define these rules by utilizing the `LinkValidationData` parameter in the link validation section of the zome. The Link Validation Data exposes the link data through an enum as well as lifecycle and package data.

