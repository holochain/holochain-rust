# holochain-nodejs

NodeJS Holochain Conductor, primarily for the execution of tests. It includes a lightweight API for orchestrating multi-agent scenario tests.

## Installation

The recommended way to install is via npm https://www.npmjs.com/package/@holochain/holochain-nodejs.

To build from source clone the repo and run
```
node ./publish.js
```
from the node folder.

You can use this built version by linking it
```
  npm link
```

Then in the project you want to use it from run
```
  npm link @holochain/holochain-nodejs
```
> Make sure you have removed it from the package.json file a a dependency before you npm install the rest of your dependencies.

## Usage

See here for details: [https://developer.holochain.org/guide/latest/intro_to_testing.html](https://developer.holochain.org/guide/latest/intro_to_testing.html).

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
- Michael Dougherty <michael.dougherty@holo.host>

## Acknowledgments

- Thanks to IronCoreLabs for the example of deploying neon modules via npm (https://github.com/IronCoreLabs/recrypt-node-binding)

## Contribute
Holochain is an open source project.  We welcome all sorts of participation and are actively working on increasing surface area to accept it.  Please see our [contributing guidelines](../CONTRIBUTING.md) for our general practices and protocols on participating in the community.

## License
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Copyright (C) 2019, Holochain Foundation

This program is free software: you can redistribute it and/or modify it under the terms of the license p
rovided in the LICENSE file (GPLv3).  This program is distributed in the hope that it will be useful, bu
t WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
 PURPOSE.

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are using GPL while we're getting the matter sorted out.  See [this article](https://medium.com/holochain/licensing-needs-for-truly-p2p-software-a3e0fa42be6c) for some of our thinking on licensing for distributed application frameworks.
