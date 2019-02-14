# Calling Zome Functions

### `conductor.call(instanceId, zomeName, functionName, callParams)` => `object`

A `Conductor` instance allows making calls to the custom functions defined the Zomes of DNA instances that it is running.
This is necessary in order to be able to test them. It calls synchronously and returns the result that the Zome function provides. An error could also be thrown, or returned.

Note that Holochain has to serialize the actual arguments for the
function call into JSON strings, which the Conductor instance will handle for you automatically. It also parses the result from a JSON string into an object. If for some reason you don't want it to do that, do `conductor._callRaw` instead, and pass a string as the fourth argument instead of an object.

This function will only succeed if `conductor.start()` has been called before it.
___
**Name** instanceId

**Type** `string`

**Description** Specifies the instance within which to call this function, by its instanceId. This instanceId should be the equivalent thing as an `instanceConfig.name` which was passed to [Config.instance](./testing_configuration.md#instances). This in turn would be equivalent to the original name given to [Config.agent](./testing_configuration.md#agents), unless you overrode it when calling [Config.instance](./testing_configuration.md#instances). See more [here](./testing_configuration.md#example-2).
___
**Name** zomeName

**Type** `string`

**Description** The name of the Zome within that instance being called into
___
**Name** functionName

**Type** `string`

**Description** The name of the custom function in the Zome to call
___
**Name** callParams

**Type** `object`

**Description** An object which will get stringified to a JSON string, before being passed into the Zome function. The keys of this object must match one-to-one with the names of the arguments expected by the Zome function, or an error will occur.
___

#### Example
```javascript
const agentConfig = Config.agent('alice')
const dnaConfig = Config.dna('path/to/bundle.json')
const instanceConfig = Config.instance(agentConfig, dnaConfig)
const conductorConfig = Config.conductor([instanceConfig])
const conductor = new Conductor(conductorConfig)
conductor.start()
const callResult = conductor.call('alice', 'people', 'create_person', {name: 'Franklin'})
```

> Note that there are some cases where, for the purposes of testing, you may wish to wait for the results of calling a
function in one instance, in terms of chain actions like commits and linking, to propogate to the other instances. For this,
extra ways of performing calls have been added as utilities. Check them out in [handling asynchronous network effects](./handling_async.md).

