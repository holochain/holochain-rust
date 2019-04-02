# DNA Instances

`DnaInstance` is a class that is exported from `holochain-nodejs` and can be imported into your code.
This class is used externally and instances of it are built automatically for you to use, so you typically should not have to construct a `DnaInstance` yourself.

A `DnaInstance` represents a running version of a DNA package by a particular agent. This means that the agent has a source chain for this DNA.
In addition to these basic properties on a DnaInstance that are covered below, the following articles cover [how to make function calls into the Zomes](./nodejs_calling_zome_functions.md).

#### Import Example
```javascript
const { DnaInstance } = require('@holochain/holochain-nodejs')
```

## Instantiate A DnaInstance

### `constructor(instanceId, conductor)` => `DnaInstance`

Instantiate a `DnaInstance` based on an instanceId, and the conductor where an instance with that id is running.
Calling this manually is not typically necessary, since the `Scenario` testing returns these natively.
A `DnaInstance` can make calls via that Conductor into Zome functions.

___
**Name** instanceId

**Type** `string`

**Description** The instance id of the DnaInstance as specified in the configuration of `conductor`. 
Note that when using the [Config.instance](./testing_configuration.md#instances) helper, the instance ID defaults to the agent name (as specified in [Config.agent](./testing_configuration.md#agents)) if not explicitly passed as a third argument.
___
**Name** conductor

**Type** `Conductor`

**Description** A valid, and running Conductor instance
___

#### Example
```javascript

const aliceInstance = new DnaInstance('alice', conductor)
```

## DnaInstance Attributes

### `dnaInstance.agentId`

The agentId for an instance.

#### Example
```javascript
console.log(alice.agentId)
// alice-----------------------------------------------------------------------------AAAIuDJb4M
```

### `dnaInstance.dnaAddress`

The address of the DNA for an instance.

#### Example
```javascript
console.log(alice.dnaAddress)
// QmYiUmMEq1WQmSSjbM7pcLCy1GkdkfbwH5cxugGmeNZPE3
```
