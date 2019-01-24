# Containers

Containers were first introduced [here](./zome/zome_functions.md#introducing-containers), when discussing Zome functions.

holochain_container

holochain-nodejs




## Testing HTTP interface using cURL

Currently the container supports the `websocket` and `http` interfaces.
Assuming the container http interface is running on port 4000 it can be tested by running:
`curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":"0","method":"info/instances"}' http://localhost:4000`
