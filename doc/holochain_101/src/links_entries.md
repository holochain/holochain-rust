# Links Entries

A link consists of 4 parts : 
`Base` - which is an Address type that corresponds to base address that we will use to connect the data
`Link Type` - which is a String Type that corresponds to a value that we would use to give the link a type
`Tag` - this is a runtime String Value that can be used to set this link to a certain tag
`Target` - this is a value that the base address with connect to through linking

The process of `linking` in holochain is done through the `LinkEntries` function in the HDK, this will allow the zome developer to connect different kinds of data. All data that is linked is stored in our `EAV storage`(see holochain-persistance-api for more details on how this works). The `EAV` is the backbone of our storage mechanism when it comes to our linking process and addition and retrival of links is done using it. 

# Validation

`System Validation`
They are two layers of validation when it comes to linking. One layer happens at the system level and this executes everytime whenever a linking is done and uses validation rules which are defined by the system. Here are some validations that can take place whenever a LinkEntry is executed. The main one is the validation of the base address when it does not exist. Since this corresponds to real data on the DHT we have to make sure that the hash we are giving it is correct and corresponds to something.

`Zome Validation`

