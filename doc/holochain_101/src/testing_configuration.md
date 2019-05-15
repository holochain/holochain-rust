# Configuration

`Config` is an object with helper functions for configuration that is exported from `holochain-nodejs` and can be imported into your code. The functions can be combined to produce a valid configuration object to instantiate a Conductor instance with.

#### Import Example
```javascript
const { Config } = require('@holochain/holochain-nodejs')
```

## Agent

### `Config.agent(agentName)` => `object`

Takes an agent name and creates a simple configuration object for that agent
___
**Name** agentName

**Type** `string`

**Description** An identifying string for this agent
___

#### Example
```javascript
const agentConfig = Config.agent('alice')
console.log(agentConfig)
/*
{
    name: 'alice'
}
*/
```

## DNA

### `Config.dna(dnaPath, [dnaName])` => `object`

Takes a path to a valid DNA package, and optionally a name and creates a simple configuration object for that DNA
___
**Name** dnaPath

**Type** `string`

**Description** The path to a `.dna.json` file containing a valid DNA configuration
___
**Name** dnaName *Optional*

**Type** `string`

**Description** The path to a `.dna.json` file containing a valid DNA configuration

**Default** The same as the given `dnaPath`
___

#### Example
```javascript
const dnaConfig = Config.dna('path/to/your.dna.json')
console.log(dnaConfig)
/*
{
    path: 'path/to/your.dna.json',
    name: 'path/to/your.dna.json'
}
*/
```

## Instances

### `Config.instance(agentConfig, dnaConfig, [name])` => `object`

Takes an agent config object and a dna confid object, and optionally a unique name, and returns a full configuration object
for a DNA instance.
___

**Name** agentConfig

**Type** `object`

**Description** A config object with a `name` property, as produced by `Config.agent`
___

**Name** dnaConfig

**Type** `object`

**Description** A config object with a `name` and `path` property, as produced by `Config.dna`
___

**Name** name *Optional*

**Type** `string`

**Description** The name acts like the instance ID, and in fact will be used as such when [calling Zome functions](./nodejs_calling_zome_functions.md)

**Default** The same as the `name` property of the given `agentConfig` (`agentConfig.name`)
___

#### Example
```javascript
const agentConfig = Config.agent('alice')
const dnaConfig = Config.dna('path/to/your.dna.json')
const instanceConfig = Config.instance(agentConfig, dnaConfig)
console.log(dnaConfig)
/*
{
    agent: {
        name: 'alice'
    },
    dna: {
        path: 'path/to/your.dna.json',
        name: 'path/to/your.dna.json'
    },
    name: 'alice'
}
*/
```

## Bridges

### `Config.bridge(handle, callerInstanceConfig, calleeInstanceConfig)` => `object`

Takes three arguments: the bridge handle, the caller, and the callee (both instances)

___

**Name** handle

**Type** `string`

**Description** The desired bridge handle, which is used by the "caller" DNA to refer to the "callee" DNA. See the [bridging section of the docs](./bridging.md) for more detail.

___

**Name** callerInstanceConfig

**Type** `object`

**Description** A config object as produced by `Config.instance`, which specifies the instance which will be making calls over the bridge

___

**Name** calleeInstanceConfig

**Type** `object`

**Description** A config object as produced by `Config.instance`, which specifies the instance which will be receiving calls over the bridge

___

#### Example
```javascript
const agentConfig1 = Config.agent('alice')
const agentConfig2 = Config.agent('bob')
const dnaConfig = Config.dna('path/to/your.dna.json')
const instanceConfig1 = Config.instance(agentConfig1, dnaConfig)
const instanceConfig2 = Config.instance(agentConfig2, dnaConfig)
const bridgeConfig = Config.bridge('bridge-handle', instanceConfig1, instanceConfig2)
console.log(bridgeConfig)
/*
{ handle: 'bridge-handle',
  caller_id: 'alice',
  callee_id: 'bob' }
*/
```


## DPKI

### `Config.dpki(instanceConfig, initParams)` => `object`

Takes two arguments: an instance object, as specified by `Config.instance`, and an object which gets passed into the `init_params` conductor config object.

___

**Name** instanceConfig

**Type** `object`

**Description** A config object with a `name` property, as produced by `Config.instance`

___

**Name** initParams

**Type** `object`

**Description** A config object which will be passed directly through to the conductor config (as `dpki.init_params`)


## Full Conductor Configuration

### `Config.conductor(conductorOptions)` => `object`

### `Config.conductor(instancesArray, [conductorOptions])` => `object`

There are two ways to construct a valid Conductor configuration from these `Config` helpers. Using the first way, you put all the config data into a single required object. Using the second "shorthand" style, you specify an array of `Config.instance` data, along with an optional object of extra options. The second way can be more convenient when you are just trying to set up a collection of instances with nothing extra options.

Consumes an array of configured instances and produces an object which is a fully valid Conductor configuration. It can be passed into the Conductor constructor, which is covered in the next articles.

> This function is mostly useful in conjunction with [manually instantiating a Conductor](./managing_the_conductor.md#instantiating-a-conductor).

___

**Name** conductorOptions *Optional*

**Type** `object`

**Description** *conductorOptions.instances* `array` Pass in an array of instance configuration objects generated by `Config.instance` to have them within the final configuration to be instantiated by the Conductor. *Note:* If using the two-argument "shorthand" style of `Config.conductor`, the first `instancesArray`
argument will override this property.

**Description** *conductorOptions.bridges* `array` Pass in an array of instance configuration objects generated by `Config.bridges` to have them within the final configuration to be instantiated by the Conductor

**Description** *conductorOptions.debugLog* `boolean` Enables debug logging. The logger produces nice, colorful output of the internal workings of Holochain.

**Default** `{ debugLog: false }`

___

**Name** instancesArray

**Type** `array`

**Description** When using the two-argument "shorthand" style of `Config.conductor`, you can specify the list of instances as the first argument, rather than folding it into the `conductorOptions` object.

___

#### Example
```javascript
const agentConfig = Config.agent('alice')
const dnaConfig = Config.dna('path/to/your.dna.json')
const instanceConfig = Config.instance(agentConfig, dnaConfig)
const conductorConfig = Config.conductor({
    instances: [instanceConfig]
})
```

Or, equivalently, using the shorthand style:

```javascript
const conductorConfig = Config.conductor([instanceConfig])
```

#### Example With conductorOptions
```javascript
const agentConfig = Config.agent('alice')
const dnaConfig = Config.dna('path/to/your.dna.json')
const instanceConfig = Config.instance(agentConfig, dnaConfig)
const conductorConfig = Config.conductor({
    instances: [instanceConfig],
    debugLog: true
})
```

Or, equivalently, using the shorthand style:

```javascript
const conductorConfig = Config.conductor([instanceConfig], {debugLog: true})
```

## Multiple Instances Example, with Bridges

```javascript
const { Config } = require('@holochain/holochain-nodejs')

// specify two agents...
const aliceName = "alice"
const bobName = "bob"
const agentAlice = Config.agent(aliceName)
const agentBob = Config.agent(bobName)
// ...and one DNA...
const dnaPath = "path/to/happ.dna.json"
const dna = Config.dna(dnaPath)
// ...then make instances out of them...
const instanceAlice = Config.instance(agentAlice, dna)
const instanceBob = Config.instance(agentBob, dna)

const bridgeForward = Config.bridge('bridge-forward', instanceAlice, instanceBob)
const bridgeBackward = Config.bridge('bridge-backward', instanceAlice, instanceBob)

// ...and finally throw them all together
const config = Config.conductor({
    instances: [instanceAlice, instanceBob],
    bridges: [bridgeForward, bridgeBackward]
})
```


