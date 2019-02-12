# Configuration

import the Config helpers and the Conductor class
```javascript
const { Config, Conductor } = require('@holochain/holochain-nodejs')
```

specify an agent...
```javascript
const agentAlice = Config.agent('alice')
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

The produced `config` is a fully valid Conductor configuration. It can be passed into the Conductor constructor, which is covered in the next article.
```javascript
const config = Config.conductor([instanceAlice])
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