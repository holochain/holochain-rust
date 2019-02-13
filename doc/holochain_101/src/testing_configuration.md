# Configuration

`Config` is an object with helper functions for configuration that is exported from `holochain-nodejs` and can be imported into your code. The functions can be combined to produce a valid configuration object to instantiate a Conductor instance with.

#### Import Example
```javascript
const { Config } = require('@holochain/holochain-nodejs')
```

## Agent

### `Config.agent(agentName)` => `object`

?? desc
___
**Name** agentName

**Type** `string`

**Description** An identifying string for this agent
___

#### Example
```javascript
const agentAlice = Config.agent('alice')
```

## DNA

### `Config.dna(dnaPath)` => `object`

?? desc
___
**Name** dnaPath

**Type** `string`

**Description** The path to a `bundle.json` file containing a valid DNA configuration
___

#### Example
```javascript
const dna = Config.dna('path/to/bundle.json')
```

## Instances

### `Config.instance(agentConfig, dnaConfig)` => `object`

?? desc
___
**Name** agentConfig

**Type** `object`

**Description** 
___
**Name** dnaConfig

**Type** `object`

**Description** 
___

#### Example
```javascript
const instanceAlice = Config.instance(agentAlice, dna)
```

## Full Conductor Configuration

### `Config.conductor(instancesList, [conductorOptions])` => `object`

Consumes an array of configured instances and produces an object which is a fully valid Conductor configuration. It can be passed into the Conductor constructor, which is covered in the next articles.

___
**Name** instancesList

**Type** `array`

**Description** 
___
**Name** conductorOptions *Optional*

**Type** `object`

**Description** 
___

#### Example
```javascript
const config = Config.conductor([instanceAlice])
```

#### Example With Opts
```javascript
const config = Config.conductor([instanceAlice], { debugLog: false })
```


## Multi Agent Example

```javascript
const { Config, Conductor } = require('@holochain/holochain-nodejs')

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
const config = Config.conductor([instanceAlice, instanceBob])
```