# HTTP

Any coding language, or tool, which can make HTTP requests can make requests to a running DNA instance. Based on the API exposed by Holochain, these must be `POST` requests, use the "application/json" Content-Type, and follow the JSON-RPC standard.

The HTTP example below will demonstrate how easy it is to make calls to a running DNA instance, just using the cURL tool for HTTP requests from a terminal.

Any of these methods could be similarly called from whatever client you are using, whether that is JS in the browser, nodejs, Ruby or any other language. For maximum ease of use, we recommend searching for a JSON-RPC helper library for your language of choice, there are lots of good ones out there.

## Starting an HTTP Server with `hc run`

`hc run --interface http`

## Starting an HTTP Server with `holochain`

To review how to start an HTTP Server with `holochain`, review the [interfaces](./conductor_interfaces.md#interfacedrivertype-enum) article.

## HTTP Example

This whole example assumes that one of the methods listed above has been used to start an HTTP server on port 8888 with a valid DNA instance running in it.

### info/instances
The following is a starter example, where a special utility function of Holochain is called, which accepts no parameters, and returns an array of the instances which are available on the HTTP server.

In another terminal besides the server, we could run the following cURL command:
`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0","id": "0","method": "info/instances"}' http://localhost:8888`

A response something like the following might be returned:
```json
{
    "jsonrpc": "2.0",
    "result": [{"id":"test-instance","dna":"hc-run-dna","agent":"hc-run-agent"}],
    "id": "0"
}
```

### Calling Zome Functions

The following discusses how to use cURL (and thus HTTP generally) to make calls to Zome functions.

The JSON-RPC "method" to use is simply "call".

The instance ID (as seen in the `info/instances` example), the Zome name, and the function name all need to be given as values in the "params" value of the JSON-RPC, in addition to the arguments to pass that function. This part of the "params" object might look like this:
`{"instance_id": "test-instance", "zome": "blog", "function": "create_post"}`

Unlike `info/instances`, Zome functions usually expect arguments. To give arguments, a JSON object should be constructed, and given as `"args"` key of the "params" value. It may look like the following:
`"args": {"content": "sample content"}`

Combining these, a request like the following could be made via cURL from a terminal:
`curl -X POST -H "Content-Type: application/json" -d '{"id": "0", "jsonrpc": "2.0", "method": "call", "params": {"instance_id": "test-instance", "zome": "blog", "function": "create_post", "args": { "content": "sample content"} }}' http://127.0.0.1:8888`

A response like the following might be returned:
```json
{
    "jsonrpc": "2.0",
    "result": "{\"Ok\":\"QmUwoQAtmg7frBjcn1GZX5fwcPf3ENiiMhPPro6DBM4V19\"}",
    "id": "0"
}
```

This response suggests that the function call was successful ("Ok") and provides the DHT address of the freshly committed blog entry ("QmU...").

This demonstrates how easy it is to call into Zome function from clients and user interfaces!
