# holochain-nodejs

Nodejs Holochain Container primarily for the execution of tests

## Installation

The recommended way to install is via npm https://www.npmjs.com/package/@holochain/holochain-nodejs.

To build from source clone the repo and run
```
node ./publish.js
```
from the project root.

## Usage
After installing via npm the module can be used in a node script as follows:

```javascript
const Container = require('@holochain/holochain-nodejs');
const app = Container.loadAndInstantiate("path/to/happ.hcpkg");
app.start();

// make calls to the app instance
// zome functions can be called using
// app.call(zome, capability, function, params);

app.stop();
```

## Deployment
Recommended pattern for deployment:

In your CLI, navigate to the directory containing these files.

Use `npm version [patch, minor, major]` (depending on the type of update)
This will update the package.json.

Commit this.

Push it to github.

Create a tag on github of the format `holochain-nodejsY.Y.Y` where `Y.Y.Y` is the version number of the tag. This is really important.

This will cause the CI to build for all platforms, and upload the binaries to github releases.

If are added as a team member on the holochain team on npm, and have previously run `npm adduser`, skip this step.
If you haven't, run `npm adduser`.
Use the details of your npm user to login.

Once travis has finished with the binary uploads to releases (which can be found at https://travis-ci.com/holochain/holochain-nodejs) run the following from your computer, from the nodejs_holochain directory
`node ./publish.js --publish`

Until windows for travis can utilize secure environment variables without breaking (its not available as a feature yet), we cannot re-enable the automated npm publish step. When the time comes, the configuration is already in the travis file, commented out.

## Authors

- Julian Laubstein <contact@julianlaubstein.de>
- Willem Olding <willem.olding@holo.host>

## Acknowledgments

- Thanks to IronCoreLabs for the example of deploying neon modules via npm (https://github.com/IronCoreLabs/recrypt-node-binding)
