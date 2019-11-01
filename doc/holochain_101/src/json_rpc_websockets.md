# WebSockets

Any coding language which has WebSockets support can communicate with the WebSocket server interface for Holochain. Based on the API exposed by Holochain, the messages must follow the JSON-RPC standard.

We recommend searching for a JSON-RPC Websockets library for the language of your choice. In this example, we will use a Javascript based JSON-RPC library.

## Starting a WebSocket Server with `hc run`

`hc run`

## Starting a WebSocket Server with `holochain`

To review how to start a WebSocket Server with `holochain`, check out the [interfaces](./conductor_interfaces.md#interfacedrivertype-enum) article.

## WebSocket Example

This whole example assumes that one of the methods listed above has been used to start a WebSocket server on port 8888 with a valid DNA instance running in it.

The JavaScript JSON-RPC library this example will use is [rpc-websockets](https://www.npmjs.com/package/rpc-websockets).

The overall pattern this example illustrates should be very similar for other languages.

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

### info/instances

The following is a starter example, where a special utility function of Holochain is called, which accepts no parameters, and returns an array of the instances which are available on the WebSocket server.

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

### Calling Zome Functions
The following discusses how to use `rpc-websockets` to make calls to Zome functions.

The JSON-RPC "method" to use is simply "call".

The instance ID (as seen in the `info/instances` example), the Zome name, and the function name all need to be given as values in the "params" value of the JSON-RPC, in addition to the arguments to pass that function. This part of the "params" object might look like this:
`{"instance_id": "test-instance", "zome": "blog", "function": "create_post"}`

 Unlike `info/instances`, Zome functions usually expect arguments. To give arguments, a JSON object should be constructed, and given as `"args"` key of the "params" value. It may look like the following:
`{ blog: { content: "sample content" }}`

The following code shows how to use `rpc-websockets` to call Zome functions.
```js
...
ws.on('open', function() {
    let method = 'call'
    let params = {
        instance_id: "test-instance",
        zome: "blog",
        function: "create_post",
        args: {
            content: "sample content"
        }
    }

    // call an RPC method with parameters
    ws.call(method, params).then(result => {
        console.log(result)
    })
})
```

If this code was run in nodejs, the output should be:
```shell
{ "Ok": "QmRjDTc8ZfnH9jucQJx3bzK5Jjcg21wm5ZNYAro9N4P7Bg" }
```

This response suggests that the function call was successful ("Ok") and provides the DHT address of the freshly committed blog entry ("QmR...").

### Closing the WebSocket Connection

When you are done permanently with the connection, it can be closed.

```
...
// close a websocket connection
ws.close()
```

All in all, calling into Zome functions from clients and user interfaces is easy!

## hc-web-client
To make it even easier in particular for web developers, there is a simple JavaScript library called `hc-web-client` developed which wraps the `rpc-websockets` library. Find it here, with instructions on how to use it:
[hc-web-client](https://github.com/holochain/hc-web-client)

