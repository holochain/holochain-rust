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

Unlike `info/instances`, a zome function call also expects arguments. We will need to include a JSON-RPC `params` field in our RPC call.

To call a Zome function, use `"call"` as the JSON-RPC `method`, and a `params` object with four items:
1. `instance_id`: The instance ID, corresponding to the instance IDs returned by `info/instances`
2. `zome`: The name of the Zome
3. `function`: The name of the function
4. `params`: The actual parameters of the zome function call 
    - (yes, it's a little confusing that a sub-field of `params` is also named `params`, but we are using an RPC method to call a zome function, so nested parameters are inevitable!)

In the last example, the instance ID "test-instance" was returned, which can be used here as the instance ID. Say there was a Zome in a DNA called "blogs", this is the Zome name. That Zome has a function called "create_blog", that is the function name. 

> Any top level keys of the `params` field should correspond **exactly** with the name of an argument expected by the Zome method being called.

**example zome function arguments**

```json
{ "blog": { "content": "sample content" }}
```

### Example request

**example request**
```json
{
    "jsonrpc": "2.0",
    "id": "0",
    "method": "call",
    "params": {
        "instance_id": "test-instance",
        "zome": "blog",
        "function": "create_blog",
        "params": {
            "blog": {
                "content": "sample content"
            } 
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

