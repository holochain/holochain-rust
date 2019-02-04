# Intro to JSON-RPC Interfaces

The JSON-RPC interface will expose, via a port on your device, a WebSocket or an HTTP server, via which you can make function calls to the Zomes of your DNA.

## JSON-RPC
JSON-RPC is a specification for using the JSON data format in a particular way, that follows the ["Remote Procedure Call"](https://en.wikipedia.org/wiki/Remote_procedure_call) pattern. Holochain uses the [Version 2 specification](https://www.jsonrpc.org/specification) of JSON-RPC. You can see general examples of JSON-RPC [here](https://www.jsonrpc.org/specification#examples).

The format for the JSON-RPC request/response pattern is really simple. A request is a JSON object  with just a few mandatory values which must be passed.

`jsonrpc`: specifies the JSON-RPC spec this request follows. The JSON-RPC spec used by Holochain Containers is `2.0`.

`id`: specifies the ID for this particular request. This is so that the request and response can be matched, even if they get transmitted out of order.

`method`: specifies the method on the "remote" (Holochain) to call.

`params`: (optional) contains a JSON object which holds the data to be given as arguments to the method being called, if the method expects them.