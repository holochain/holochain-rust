# trycp_server

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![PM](https://img.shields.io/badge/pm-waffle-blue.svg?style=flat-square)](https://waffle.io/holochain/org)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.org)

A server for provisioning Holochain conductors on a node. Currently used by the [tryorama](https://github.com/holochain/tryorama) test orchestrator.

## Install

From the nix-shell:

``` shell
hc-trycp-server-install
```

## Usage

Start the server with:

`trycp_server -p <port>  --port-range <port_range_string>`

The --port-range option is required as it is what reports back to try-o-rama about which ports to use when generating config files.

Example usage from a nodejs script see the [test/test.js](https://github.com/holochain/holochain-rust/blob/trycp/crates/trycp_server/test/test.js) file.

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
