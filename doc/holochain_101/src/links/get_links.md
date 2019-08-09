# Get Links
Get Links allows the zome developer to query links from the DHT. The call accepts an options parameter to customize the query behavior. The parameters of the get_links are : 

`base` :  address of the entry on which to query for links
`LinkType Link Match` : a match enum which is either a regex or an exact match specifier on link_type
`Tag Link Match` : a match enum which is either a regex or an exact match specifier on link's tag
`Options` : a struct (see below) that you can use to specify different options to apply when executing the query.

# Options
`Timeout` : The timeout variable on the options specifies how long the query process should wait befor a response before it timesout
`LinksStatusRequest` : This is a variable in which you can specify 3 modes, `All`,`Live`,`Delete`. This allows you to query the links based on crud_status in which `All` will return everything will `Live` will only return live links and `Delete` as such.
`Headers`: boolean value which if set to true indicates that the link headers should also be returned.```


# Link Results

A successful get_links call returns a set of links [(base,link_type,tag,target)] as well as meta data associated with it which is the headers and crud_status if requested in the `Options`.  
 
