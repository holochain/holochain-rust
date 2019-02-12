# Configuration



import the Config helpers and the Conductor class
```javascript
const { Config, Conductor } = require('@holochain/holochain-nodejs')
```

specify an agent...
```javascript
const aliceName = "alice"
const agentAlice = Config.agent(aliceName)
```

one DNA
```javascript
const dnaPath = "path/to/happ.hcpkg"
const dna = Config.dna(dnaPath)
```

Make instances out of them
```javascript
const instanceAlice = Config.instance(agentAlice, dna)
```

The produced `config` is a fully valid Conductor configuration
```javascript
const config = Config.conductor([instanceAlice])
```

A configuration object is passed into the Conductor constructor
```javascript
const conductor = new Conductor(config)
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

// The produced `config` is a fully valid Conductor configuration and can be
// passed directly to the Conductor constructor
const conductor = new Conductor(config)
conductor.start()
```