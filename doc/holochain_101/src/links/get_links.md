# Get Links
Get Links allows the zome developer to query links from the DHT. The get links can be configured with options to customize what type of query should be returned. The parameters of the get_links are : 

`base` : which is the base address
`LinkType Link Match` : which you can use to specify a regex of an exact match or pattern on the match of a link_type
`Tag Link Match` : which you can use to specify a regex of an exact match or pattern on the match of a tag
`Options` : This a configurable struct that you can use to specify different options to apply when executing the query.

# Options
`Timeout` : The timeout variable on the options specifies how long the query process should wait befor a response before it timesout
`LinksStatusRequest` : This is a variable in which you can specify 3 modes, `All`,`Live`,`Delete`. This allows you to query the links based on crud_status in which `All` will return everything will `Live` will only return live links and `Delete` as such.
`Headers`: With the headers, you will be able to specify if you should return link headers as well. This is a boolean value that can true or false


# Link Results

On a succesful get_links operation, it returns a set of links [(base,link_type,tag,target)] as well as meta data associated with it which is the headers and crud_status if specified. Even though the link is stored by associating it with the has of the LinkAdd entry in our `EAV storage` what comes back is a link target.  
 
