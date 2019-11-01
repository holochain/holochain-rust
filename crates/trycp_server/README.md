# Try-o-Rama Control Protocol tools (trycp)

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![PM](https://img.shields.io/badge/pm-waffle-blue.svg?style=flat-square)](https://waffle.io/holochain/org)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.org)

An implementation of the Try-o-rama Control Protocol using simple bash conductor manager with a JsonRPC server wrapper.

## Install

From the nix-shell:

``` shell
hc-trycp-server-install
```

## Usage

Start RPC the server with:

`trycp_server`
or
`trycp_server -p <port>`


Example usage from a nodejs script:

``` javascript
var WebSocket = require('rpc-websockets').Client

var ws = new WebSocket('ws://localhost:9000')
console.log("starting")
ws.on('open', function() {
    console.log("making ping call")

    ws.call('ping', {"id": "my-player"}).then(function(result) {
        console.log(result)
    })

    const config_toml = "[config]"  // note that this is not a valid config so spawn will fail
    const config = Buffer.from(config_toml).toString('base64')
    console.log("making player call")
    ws.call('player', {"id": "my-player", "config": config}).then(function(result) {
        console.log(result)
    })

    console.log("making spawn call")
    ws.call('spawn', {"id": "my-player"}).then(function(result) {
        console.log(result)
    })

    console.log("making kill call")
    ws.call('kill', {"id": "my-player"}).then(function(result) {
        console.log(result)
        ws.close()
    })

    // close a websocket connection
    //ws.close()
})
```

## Docker

This is all intended to run from inside many docker boxes all floating around on the internet somewhere.

There are some nix commands to help make this work.

`hc-trycp-docker-build`

Rebuild the docker box.

Is slower to build if the underlying box is stale.

The underlying box is `holochain/holochain-rust:latest`.

`hc-trycp-docker-run`

Runs the docker box detached.

`trycp_server` is run on port 443 internally.

Maps the internal port 443 to the host port 443.

`hc-trycp-docker-attach`

Attaches the already-running docker box.

Useful for local debugging.

## License
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Copyright (C) 2019, Holochain Foundation

This program is free software: you can redistribute it and/or modify it under the terms of the license p
rovided in the LICENSE file (GPLv3).  This program is distributed in the hope that it will be useful, bu
t WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
 PURPOSE.

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are using GPL while we're getting the matter sorted out.  See [this article](https://medium.com/holochain/licensing-needs-for-truly-p2p-software-a3e0fa42be6c) for some of our thinking on licensing for distributed application frameworks.
