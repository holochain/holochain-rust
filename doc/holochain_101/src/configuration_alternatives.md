<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Contents**

- [Configuration Alternatives](#configuration-alternatives)
      - [Using a Plain Old Javascript Object](#using-a-plain-old-javascript-object)
      - [Using TOML](#using-toml)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

# Configuration Alternatives

It is possible to use the same configuration as you would for the [`holochain` Conductor](./production_conductor.md), and pass it to the constructor for `Conductor`. The configuration may be a string of valid TOML, or a JavaScript object with the equivalent structure. To review the configuration, [go here](./intro_to_toml_config.md).

To see some examples of what these configuration files can look like, you can check out [this folder on GitHub](https://github.com/holochain/holochain-rust/tree/develop/conductor/example-config).

#### Using a Plain Old Javascript Object

```javascript
const { Conductor } = require('@holochain/holochain-nodejs')
const conductor = new Conductor({
    agents: [],
    dnas: [],
    instances: [],
    bridges: [],
    // etc...
})
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
```

