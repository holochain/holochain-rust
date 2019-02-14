# Managing the Conductor

`Conductor` is a class that is exported from `holochain-nodejs`, and can be imported into your code.

#### Import Example
```javascript
const { Conductor } = require('@holochain/holochain-nodejs')
```

## Instantiating a Conductor

### `constructor(conductorConfig)` => `Conductor`

Instantiate a Conductor with a Conductor configuration, obtained by using one of the approaches outlined in the [configuration articles](./testing_configuration.md).

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

## Starting and Stopping a Conductor

### `conductor.start()` => null

Start running all instances. No Zome functions can be called within an instance if the instance is not started, so this must be called beforehand.

#### Example
```javascript
conductor.start()
```

### `conductor.stop()` => `Promise`

Stop all running instances configured for the conductor. This function **should** be called after all desired Zome calls have been made, otherwise the conductor instances will continue running as processes in the background.

Returns a Promise that you can optionally wait on, which will wait for internal cleanup to happen prior to shutting down.

#### Example
```javascript
conductor.stop()
```
