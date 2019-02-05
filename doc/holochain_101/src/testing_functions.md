# Testing Functions

The purpose of the `holochain-nodejs` module is to make integration tests and scenario tests able to be written as simply and with as little boilerplate as possible. However, the module also provides even more basic functionality, making it possible to build tests with whatever tradeoff between convenience and customization is right for your project.

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

// The produced `config` is a fully valid conductor configuration and can be
// passed directly to the conductor constructor
const conductor = new Conductor(config)
conductor.start()
```

## Using the conductor

```javascript
const conductor = new Conductor(config)
conductor.start()

// When building up a config using `Config`, the instance ID is automatically assigned
// as the given agent ID plus a double colon plus the given dnaPath.
// We'll need this to call the instance later.
const aliceInstanceId = aliceName + '::' + dnaPath

// zome functions can be called using the following, assuming the vars are defined with valid values
const callResult = conductor.call(aliceInstanceId, zome, capability, fnName, paramsAsObject)
// the same could be accomplished using the following, makeCaller is for convenience
const alice = conductor.makeCaller(aliceName, dnaPath)
const altCallResult = alice.call(zome, capability, fnName, paramsAsObject)

// get the actual agent_id for an instance, by passing an instance id
const aliceAgentId = conductor.agent_id(aliceInstanceId)

// stop all running instances
conductor.stop()
```

### Configuration Alternatives

Simply use the same configuration as you would for `holochain_conductor`, and pass it to the constructor for `Conductor`. The configuration may be a string of valid TOML, or a Javascript object with the same structure

#### Using a Javascript Object

```javascript
const { Conductor } = require('@holochain/holochain-nodejs')
const conductor = new Conductor({
    agents: [],
    dnas: [],
    instances: [],
    bridges: [],
    // etc...
})

conductor.start()
// do what you will
conductor.stop()
```

#### Using TOML

```javascript
const { Conductor } = require('@holochain/holochain-nodejs')
const toml = `
[[agents]]
<agent config>

[[dnas]]
<dna config>

[[instances]]
...etc...
`
const conductor = new Conductor(toml)

conductor.start()
// do what you will
conductor.stop()
```

