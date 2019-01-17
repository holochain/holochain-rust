# holochain-nodejs

NodeJS Holochain Container, primarily for the execution of tests. It includes a lightweight API for orchestrating multi-agent scenario tests.

## Installation

The recommended way to install is via npm https://www.npmjs.com/package/@holochain/holochain-nodejs.

To build from source clone the repo and run
```
node ./publish.js
```
from the project root.

## Basic Usage

The following demo shows how to spin up two separate instances of a DNA, within the container.

After installing via npm the module can be used in a node script as follows:
```javascript
const dnaPath = "path/to/happ.hcpkg"
const aliceName = "alice"
const tashName = "tash"
// destructure to get Config and Container off the main import, which is an object now
const { Config, Container } = require('@holochain/holochain-nodejs')

// build up a configuration for the container, step by step
const agentAlice = Config.agent(aliceName)
const agentTash = Config.agent(tashName)
const dna = Config.dna(dnaPath)
const instanceAlice = Config.instance(agentAlice, dna)
const instanceTash = Config.instance(agentTash, dna)
const config = Config.container([instanceAlice, instanceTash])

// create a new instance of a Container, from the config
const container = new Container(config)

// this starts all the configured instances
container.start()

// When building up a config using `Config`, the instance ID is automatically assigned
// as the given agent ID plus a double colon plus the given dnaPath.
// We'll need this to call the instance later.
const aliceInstanceId = aliceName + '::' + dnaPath

// zome functions can be called using the following, assuming the vars are defined with valid values
const callResult = container.call(aliceInstanceId, zome, fnName, paramsAsObject)
// the same could be accomplished using the following, makeCaller is for convenience
const alice = container.makeCaller(aliceName, dnaPath)
const altCallResult = alice.call(zome, fnName, paramsAsObject)

// get the actual agent_id for an instance, by passing an instance id
const actualAliceAgentId = container.agent_id(aliceInstanceId)

// stop all running instances
container.stop()
```


## Deployment
Recommended pattern for deployment:

In your CLI, navigate to the directory containing these files.

Use `npm version [patch, minor, major]` (depending on the type of update)
This will update the package.json.

Commit this.

Push it to github.

Create a tag on github of the format `holochain-nodejs-vY.Y.Y` where `Y.Y.Y` is the version number of the tag. This is really important, as only a tag with this format will trigger release builds to happen. This is configured in the .travis.yml file.

This will cause the CI to build for all platforms, and upload the binaries to github releases.

If are added as a team member on the holochain team on npm, and have previously run `npm adduser`, skip this step.
If you haven't, run `npm adduser`.
Use the details of your npm user to login.

Once travis has finished with the binary uploads to releases (progress can be seen at https://travis-ci.org/holochain/holochain-rust) run the following from your computer, from the `nodejs_holochain` directory
`node ./publish.js --publish`

Until windows for travis can utilize secure environment variables without breaking (its not available as a feature yet), we cannot re-enable the automated npm publish step. When the time comes, the configuration is already in the travis file, commented out.

## Authors

- Julian Laubstein <contact@julianlaubstein.de>
- Connor Turland <connor.turland@holo.host>
- Willem Olding <willem.olding@holo.host>

## Acknowledgments

- Thanks to IronCoreLabs for the example of deploying neon modules via npm (https://github.com/IronCoreLabs/recrypt-node-binding)

## Contribute
Holochain is an open source project.  We welcome all sorts of participation and are actively working on increasing surface area to accept it.  Please see our [contributing guidelines](../CONTRIBUTING.md) for our general practices and protocols on participating in the community.

## License
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Copyright (C) 2018, Holochain Trust

This program is free software: you can redistribute it and/or modify it under the terms of the license p
rovided in the LICENSE file (GPLv3).  This program is distributed in the hope that it will be useful, bu
t WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
 PURPOSE.

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are using GPL while we're getting the matter sorted out.  See [this article](https://medium.com/holochain/licensing-needs-for-truly-p2p-software-a3e0fa42be6c) for some of our thinking on licensing for distributed application frameworks.
