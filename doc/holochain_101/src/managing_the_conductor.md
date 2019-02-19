# Managing the Conductor

`Conductor` is a class that is exported from `holochain-nodejs`, and can be imported into your code.
It is mostly used internally in the library, but can be useful in some of your own use cases.

#### Import Example
```javascript
const { Conductor } = require('@holochain/holochain-nodejs')
```

## Simple Use

### `Conductor.run(conductorConfig, runner)` => `Promise`

Spin up a Conductor with a [Conductor configuration](./testing_configuration.md). When you're done with it, call `stop`, a function injected into the closure.

___
**Name** conductorConfig

**Type** `string` or `object`

**Description** should be a TOML configuration string, as described [here](./configuration_alternatives.md) or an equivalent JavaScript object constructed manually, or setup using the `Config` helper functions described [here](./testing_configuration.md).
___
**Name** runner

**Type** `function`

**Description** `runner` is a closure: (stop, conductor) => { (code to run) }
- `stop` is a function that shuts down the Conductor and must be called in the closure body
- `conductor` is a Conductor instance, from which one can make [Instances](./nodejs_instances.md) and thus Zome calls.
___

#### Example
```javascript
// ...
Conductor.run(Config.conductor([
    instanceAlice,
    instanceBob,
    instanceCarol,
]), (stop, conductor) => {
    doStuffWith(conductor)
    stop()
})
```

## Manually Instantiating a Conductor

### `constructor(conductorConfig)` => `Conductor`

Instantiate a Conductor with a full [Conductor configuration](./testing_configuration.md).

___
**Name** conductorConfig

**Type** `string` or `object`

**Description** should be a TOML configuration string, as described [here](./configuration_alternatives.md) or an equivalent JavaScript object constructed manually, or setup using the `Config` helper functions described [here](./testing_configuration.md).
___

#### Example
```javascript
// config var can be defined using the Config helper functions
const conductor = new Conductor(config)
```

## Manually Starting and Stopping a Conductor

### `conductor.start()` => null

Start running all instances. No Zome functions can be called within an instance if the instance is not started, so this must be called beforehand.

#### Example
```javascript
conductor.start()
```

### `conductor.stop()` => `Promise`

Stop all running instances configured for the conductor. This function **should** be called after all desired Zome calls have been made, otherwise the conductor instances will continue running as processes in the background.

Returns a Promise that you can optionally wait on to ensure that internal cleanup is complete.

#### Example
```javascript
conductor.stop()
```
