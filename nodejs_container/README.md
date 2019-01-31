# holochain-nodejs

NodeJS Holochain Container, primarily for the execution of tests. It includes a lightweight API for orchestrating multi-agent scenario tests.

## Installation

The recommended way to install is via npm https://www.npmjs.com/package/@holochain/holochain-nodejs.

To build from source clone the repo and run
```
node ./publish.js
```
from the project root.

## Usage for tests

The purpose of this module is to make integration tests and scenario tests able to be written as simply and with as little boilerplate as possible. However, the module also provides even more basic functionality, making it possible to build tests with whatever tradeoff between convenience and customization is right for your project.

Let's start with the most convenient case and see how to gain more control over things later.

### Writing simple tests with `tape`

The following example shows the simplest, most convenient way to start writing tests with this module. We'll set up an environment for running tests against two instances of one DNA, using the [tape](https://github.com/substack/tape) test harness:

```javascript
const { Config, Scenario } = require('@holochain/holochain-nodejs')

Scenario.setTape(require('tape'))

// specify two agents...
const agentAlice = Config.agent("alice")
const agentBob = Config.agent("bob")
// ...and one DNA...
const dna = Config.dna("path/to/happ.hcpkg")
// ...then make instances out of them...
const instanceAlice = Config.instance(agentAlice, dna)
const instanceBob = Config.instance(agentBob, dna)

// Now we can construct a `scenario` object which lets us run as many scenario tests as we want involving the two instances we set up:
const scenario = new Scenario([instanceAlice, instanceBob])

// Run a test in a freshly created container. Note the various parameters used:
// - a description which will be used to initialize the tape test
// - a closure that takes two arguments which will be injected:
//   - `t` is the just the object that tape tests use
//   - the second argument is an object containing an interface into each instance specified in the config
scenario.runTape("test something", (t, {alice, bob}) => {
    // fire zome function calls from both agents
    const result1 = alice.call('zome', 'capability', 'function', {params: 'go here'})
    const result2 = bob.call('zome', 'capability', 'function', {params: 'go here'})
    // make some tape assertions
    t.ok(result1)
    t.equal(result2, 'expected value')

    // when this function ends, the test is automaticaly ended,
    // and the container is stopped
})

// Run another test in a freshly created container
scenario.runTape("test something else", (t, {alice, bob}) => {
    // write more tests in the same fashion
})
```

Note that we used two objects here:

* `Config`, which was used to build up a valid configuration for the scenario tests
* `Scenario`, which does all the work of starting and stopping containers and integrating with various test harnesses (currently only `tape` is supported).

Each invocation of `scenario.runTape` does the following:

1. Starts a fresh Container based on the configuration used to construct `scenario`
2. Starts a new `tape` test
3. Injects the values needed for the test into a closure you provide:
    * `t`, which has the usual `tape` interface for assertions, etc.
    * an object which contains an interface to each of the instances specified by the config
4. Automatically ends the test and stops the container when the closure is done running

### Using other test harnesses

Only `tape` is currently supported as a fully integrated test harness, but you can also run tests with more manual control using `scenario.run`, like so:

```javascript
// this example will also use `tape` as an illustration, but each test harness would have its own particular way
const tape = require('tape')

// Create a scenario object in the same fashion as the previous example
const scenario = new Scenario([instanceAlice, instanceBob])

// scenario.run only manages the Container for us now, but we have to manage the test itself
scenario.run((stop, {alice, bob}) => {
    tape("test something", t => {
        const result = alice.call('zome', 'capability', 'function', {params: 'go here'})
        t.equal(result, 'expected value')
        // the following two steps were not necessary when using runTape:
        t.end() // end the test
        stop() // use this injected function to stop the container
    })
})

scenario.run((stop, {alice, bob}) => {
    tape("test something else", t => {
        // write more tests in the same fashion
        t.equal(2 + 2, 4)
        t.end()
        stop() // but don't forget to stop the container when it's done!
    })
})
```

Using `run` allows you to manage the test yourself, only providing you with the basic help of starting and stopping a fresh container instance.

The previous example used `tape` to show how it compares to using `runTape`, though you could have used any test harness, like Jest or Mocha. In fact, `runTape` simply calls `run` under the hood.

### Dealing with asynchronous network effects

In the previous example, we used `alice.call()` to call a zome function. This returns immediately with a value, even though the test network created by the container is still running, sending messages back and forth between agents for purposes of validation and replication, etc. In many test cases, you will want to wait until all of this network activity has died down to advance to the next step.

For instance, take the very common scenario as an example:

1. Alice runs a zome function which commits an entry, then adds a link to that entry
2. Bob runs a zome function which attempt to get links, which should include the link added by alice

If the test just uses `call()` to call that zome function, there is no guarantee that the entries committed by `alice.call` will be available on the DHT by the time `bob.call` is started. Therefore, two other functions are available.

**`alice.callSync`** returns a Promise instead of a simple value. The promise does not resolve until network activity has completed.
**`alice.callWithPromise`** is a slightly lower-level version of the same thing. It splits the value apart from the promise into a tuple  `[value, promise]`, so that the value can be acted on immediately and the promise waited upon separately.

```javascript
// If we make the closure `async`, we can use `await` syntax to keep things cleaner
scenario.run(async (stop, {alice, bob}) => {
    tape("test something", t => {
        // we can await on `callSync` immediately, causing it to block until network activity has died down
        const result1 = await alice.callSync('zome', 'cap', 'do_something_that_adds_links', {})
        // now bob can be sure he has access to the latest data
        const result2 = bob.call('zome', 'cap', 'get_those_links', {})
        t.equal(result, 'expected value')
        // the following two steps were not necessary when using runTape:
        t.end() // end the test
        stop() // use this injected function to stop the container
    })
})
```

Even though we can't solve the eventual consistency problem in real life networks, we can solve them in tests when we have total knowledge about what each agent is doing.

## Running Containers manually with `Container`

If you are using this module for something other than standard test suites, you may want more control over how containers get built.

Simply use the same configuration as you would for `holochain_container`, and pass it to the constructor for `Container`. The configuration may be a string of valid TOML, or a Javascript object with the same structure

### Using TOML

```javascript
const { Container } = require('@holochain/holochain-nodejs')
const toml = `
[[agents]]
<agent config>

[[dnas]]
<dna config>

[[instances]]
...etc...
`
const container = new Container(toml)

container.start()
// do what you will
container.stop()
```

### Using Javascript object

TODO: make sure this actually works

```javascript
const { Container } = require('@holochain/holochain-nodejs')
const container = new Container({
    agents: [],
    dnas: [],
    instances: [],
    bridges: [],
    // etc...
})

container.start()
// do what you will
container.stop()
```

## Creating valid configuration more conveniently

Note that you can build fully valid configuration similar to the above scenario test examples using the `Config` object:

```javascript
const { Config, Container } = require('@holochain/holochain-nodejs')


// specify two agents...
const aliceName = "alice"
const bobName = "bob"
const agentAlice = Config.agent(aliceName)
const agentBob = Config.agent(bobName)
// ...and one DNA...
const dnaPath = "path/to/happ.hcpkg"
const dna = Config.dna(dnaPath)
// ...then make instances out of them...
const instanceAlice = Config.instance(agentAlice, dna)
const instanceBob = Config.instance(agentBob, dna)
// ...and finally throw them all together
const config = Config.container([instanceAlice, instanceBob])

// The produced `config` is a fully valid container configuration and can be
// passed directly to the container constructor
const container = new Container(config)
container.start()
```

## Using the container

```javascript
const container = new Container(config)
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
const aliceAgentId = container.agent_id(aliceInstanceId)

// stop all running instances
container.stop()
```

container.start, container.call, container.agent_id, and container.stop are the four functions of Container instances currently.

Note about usage:
Prior to version 0.0.3, a container would only return a single instance of an app. Now a container actually contains multiple instances. When performing a call to an instance, one must include the instance id. Take the following for example:

```
const callResult = container.call(someInstanceId, someZome, someFunction, someParams)
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
- Michael Dougherty <michael.dougherty@holo.host>

## Acknowledgments

- Thanks to IronCoreLabs for the example of deploying neon modules via npm (https://github.com/IronCoreLabs/recrypt-node-binding)

## Contribute
Holochain is an open source project.  We welcome all sorts of participation and are actively working on increasing surface area to accept it.  Please see our [contributing guidelines](../CONTRIBUTING.md) for our general practices and protocols on participating in the community.

## License
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

Copyright (C) 2019, Holochain Trust

This program is free software: you can redistribute it and/or modify it under the terms of the license p
rovided in the LICENSE file (GPLv3).  This program is distributed in the hope that it will be useful, bu
t WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
 PURPOSE.

**Note:** We are considering other 'looser' licensing options (like MIT license) but at this stage are using GPL while we're getting the matter sorted out.  See [this article](https://medium.com/holochain/licensing-needs-for-truly-p2p-software-a3e0fa42be6c) for some of our thinking on licensing for distributed application frameworks.
