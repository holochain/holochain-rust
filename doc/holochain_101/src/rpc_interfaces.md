# RPC Interfaces (Websockets/HTTP)

Currently the Container supports the `websocket` and `http` interfaces.

hc-web-client

## HTTP
Assuming the Container http interface is running on port 4000 it can be tested by running:
`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":"0","method":"info/instances"}' http://localhost:4000`