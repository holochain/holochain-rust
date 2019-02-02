# JSON-RPC Interfaces (Websockets/HTTP)

There is another option for `hc run` that wasn't yet covered.

`--interface` is an option for selecting a JSON-RPC interface to serve your DNA over.

The JSON-RPC interface will expose, via a port on your device, a WebSocket or an HTTP server, via which you can make function calls to the Zomes of your DNA.

The default interface is `websocket`.

To run it as HTTP, run
`hc run --interface http`



## JSON-RPC
JSON-RPC is a specification for using the JSON data format in a particular way, that follows the ["Remote Procedure Call"](https://en.wikipedia.org/wiki/Remote_procedure_call) pattern. Holochain uses the [Version 2 specification](https://www.jsonrpc.org/specification) of JSON-RPC. You can see general examples of JSONRPC [here](https://www.jsonrpc.org/specification#examples).


## WebSockets

Any coding language which has WebSockets support can communicate with the WebSocket server interface for Holochain. Based on the API exposed by Holochain, the messages must follow the JSON-RPC standard.

We recommend searching for a JSON-RPC Websockets library for the language of your choice. In this example, we will use a Javascript based JSON-RPC library.

### WebSocket Example

The JavaScript JSON-RPC library this example will use is [rpc-websockets](https://www.npmjs.com/package/rpc-websockets).

The pattern for other languages should be very similar.

The rest of this example assumes that in one terminal, we have run the following command in a valid DNA folder (which by default runs as a WebSocket server on port 8888):
`hc run`

For nodejs, and using NPM, install the `rpc-websockets` package by running:
`npm install rpc-websockets`

The following code snippet just does the setup for interacting with your running DNA instance:
```js
// import the rpc-websockets library
let WebSocket = require('rpc-websockets').Client

// instantiate Client and connect to an RPC server
let holochainUri = 'ws://localhost:8888'
let ws = new WebSocket(holochainUri)
 
// create an event listener, and a callback, for when the socket connection opens
ws.on('open', function() {
  // do stuff in here
})
```

#### WS: info/instances

The following is a starter example, where a special utility function of Holochain is called, which accepts no parameters, and returns an array of the instances which are running behind the WebSocket server.

The name of this special method is `info/instances`. The following code shows how to use `rpc-websockets` to call it. (Note the previous code is collapsed in the ellipsis for brevity)
```js
...
ws.on('open', function() {
  
  let method = 'info/instances'
  let params = {}
  // call an RPC method with parameters
  ws.call(method, params).then(result => {
      console.log(result)
  })
})
```

If this code was run in nodejs, the output should be:
```shell
[ { id: 'test-instance', dna: 'hc-run-dna', agent: 'hc-run-agent' } ]
```

#### WS: Calling Zome Functions
As a more interesting example, the following discusses how to use `rpc-websockets` to make calls to Holochain functions.

To call a Zome function, we need four things:
1. The instance ID
2. The name of the Zome
3. The name of the function
4. The arguments for the function call

Using the JSON-RPC protocol, we combine the first three into a single string, separated by forward slash (`/`) characters, to use as the "method". For `4`, we pass that in the "params" key of a JSON-RPC call.

In the last example, the instance ID "test-instance" was returned, which would be used this as the instance ID. Now say that there was a Zome in a DNA called "blogs", this is the Zome name. Say that Zome has a function called "create_blog", that is the function name. Append these together with `/` characters to get the "method" for the JSON-RPC call.

`"test-instance/blogs/create_blog"`

Unlike `info/instances`, Zome functions usually expect arguments. To give arguments, a JSON object should be constructed. The top level keys of the input object should correspond with the name of an argument expected by the function in the Zome being called. It may look like the following:
`{"blog": {"content": "sample content"}}`

The following code shows how to use `rpc-websockets` to call Zome functions.
```js
...
ws.on('open', function() {
    let method = 'test-instance/blogs/create_blog'
    let params = {
        "blog": {
            "content": "sample content"
        }
    }
    ws.call(method, params).then(result => {
        console.log(result)
    })
})
```

If this code was run in nodejs, the output should be:
```shell
{"Ok":"QmRjDTc8ZfnH9jucQJx3bzK5Jjcg21wm5ZNYAro9N4P7Bg"}
```

This response suggests that the function call was successful ("Ok") and provides the DHT address of the freshly committed blog entry.

#### Closing the WebSocket Connection

When you are done permanently with the connection, it can be closed.

```
...
// close a websocket connection
ws.close()
```

All in all, calling into Zome functions from clients and user interfaces is easy!

### hc-web-client
To make it even easier in particular for web developers, there is a simple JavaScript library called `hc-web-client` developed which wraps the `rpc-websockets` library. Find it here, with instructions on how to use it:
[hc-web-client](https://github.com/holochain/hc-web-client)


## HTTP

Any coding language, or tool, which can make HTTP requests can make requests to a running DNA instance. Based on the API exposed by Holochain, these must be `POST` requests, use the "application/json" Content-Type, and follow the JSON-RPC standard.

The HTTP example below will demonstrate how easy it is to make calls to a running DNA instance, just using the cURL tool for HTTP requests from a terminal.

Any of these methods could be similarly called from whatever client you are using, whether that is JS in the browser, nodejs, Ruby or any other language. For maximum ease of use, we recommend searching for a JSON-RPC helper library for your language of choice, there are lots of good ones out there.

### HTTP Example

The rest of this discussion assumes that in one terminal, we have run the following command in a valid DNA folder:
`hc run --interface http --port 8888`

Since this is a long running process, we leave that terminal open, while we open other ones to continue with the following.

#### HTTP: info/instances
In another terminal, we could run the following cURL command:
`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0","id": "0","method": "info/instances"}' http://localhost:8888`

A response something like the following might be returned:
```json
{
    "jsonrpc": "2.0",
    "result": [{"id":"test-instance","dna":"hc-run-dna","agent":"hc-run-agent"}],
    "id": "0"
}
```

The formatting of this request/response pattern is really simple. `"jsonrpc": "2.0"` specifies the jsonrpc spec being adhered to. `"id": "0"` is an ID for this particular request, so that the request and response can be matched. `"method": "info/instances"` specifies the method on the "remote" (Holochain) to call.

The special method "info/instances" doesn't require any input parameters, so they're absent from this request.

What `info/instances` does is return a list (the array value of the "result" key of the response) of the running instances. For each running instance, it provides the instance "id", the name of the DNA, and the agent "id".

The instance id will be particularly useful in other circumstances.

#### HTTP: Calling Zome Functions

The first example was just a utility call, the following explains how a call to an actual Zome function can be made.

To call a Zome function, we need four things:
1. The instance ID
2. The name of the Zome
3. The name of the function
4. The arguments for the function call

Using the JSON-RPC protocol, we combine the first three into a single string, separated by forward slash (`/`) characters, to use as the "method". For `4`, we pass that in the "params" key of a JSON-RPC call.

In the last example, the instance ID "test-instance" was returned, which would be used this as the instance ID. Now say that there was a Zome in a DNA called "blogs", this is the Zome name. Say that Zome has a function called "create_blog", that is the function name. Append these together with `/` characters to get the "method" for the JSON-RPC call.

`"method": "test-instance/blogs/create_blog"`

Unlike `info/instances`, Zome functions usually expect arguments. To give arguments, a JSON object should be constructed. It may look like the following:
`"params": {"blog": {"content": "sample content"}}`

> "blog" (and any top level keys of the input object) should correspond with the name of an argument expected by the "create_blog" function in the Zome.

With all this, a request like the following could be made:
`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "id": "0", "method": "test-instance/blogs/create_blog", "params": {"item":{"content":"sample content"}}}' http://localhost:8888`

A response something like the following might be returned:
```json
{
    "jsonrpc":"2.0",
    "result":"{\"Ok\":\"QmUwoQAtmg7frBjcn1GZX5fwcPf3ENiiMhPPro6DBM4V19\"}",
    "id":"0"
}
```

This response suggests that the function call was successful ("Ok") and provides the DHT address of the freshly committed blog entry.