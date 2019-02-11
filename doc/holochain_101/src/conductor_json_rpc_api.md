# Conductor JSON-RPC API



## Querying Running DNA Instances

Holochain Conductors expose a method `info/instances`. This method returns a list of the running DNA instances in the Conductor. For each running instance, it provides the instance "ID", the name of the DNA, and the agent "id". The instance IDs will be particularly useful in other circumstances.

The method `info/instances` doesn't require any input parameters, so `params` can be left off the request.

### Example
**example request**
```json
{
    "jsonrpc": "2.0",
    "id": "0",
    "method": "info/instances"
}
```

**example response**
```json
{
    "jsonrpc": "2.0",
    "result": [{"id":"test-instance","dna":"hc-run-dna","agent":"hc-run-agent"}],
    "id": "0"
}
```

## Calling Zome Functions

The following explains the general JSON-RPC pattern for how to call a Zome function.

### method
To call a Zome function, for the `method` we need three things:
1. The instance ID, corresponding to the instance IDs returned by `info/instances`
2. The name of the Zome
3. The name of the function

The three are combined into a single string, separated by forward slash (`/`) characters, to use as the JSON-RPC "method".

In the last example, the instance ID "test-instance" was returned, which can be used here as the instance ID. Say there was a Zome in a DNA called "blogs", this is the Zome name. That Zome has a function called "create_blog", that is the function name. Append these together with `/` characters to get the "method" for the JSON-RPC call.

**example method**
```
"test-instance/blogs/create_blog"
```

### params
Unlike `info/instances`, Zome functions usually expect arguments. To give arguments, a JSON object should be constructed.

> Any top level keys of the input object should correspond **exactly** with the name of an argument expected by the Zome method being called.

**example params**
```json
{ "blog": { "content": "sample content" }}
```

### Example
**example request**
```json
{
    "jsonrpc": "2.0",
    "id": "0",
    "method": "test-instance/blogs/create_blog",
    "params": {
        "blog": {
            "content":"sample content"
        }
    }
}
```

**example response**
```json
{
    "jsonrpc": "2.0",
    "result": "{\"Ok\":\"QmUwoQAtmg7frBjcn1GZX5fwcPf3ENiiMhPPro6DBM4V19\"}",
    "id": "0"
}
```

This response suggests that the function call was successful ("Ok") and provides the DHT address of the freshly committed blog entry ("QmU...").

