# Calling Zome Functions

### `dnaInstance.call(zomeName, functionName, callParams)` => `object`

A `DnaInstance`  can use the `Conductor` in which it's running to make calls to the custom functions defined in its Zomes.
This is necessary in order to be able to test them. It calls synchronously and returns the result that the Zome function provides. An error could also be thrown, or returned.

Note that Holochain has to serialize the actual arguments for the
function call into JSON strings, which the Conductor will handle for you automatically. It also parses the result from a JSON string into an object.

This function will only succeed if `conductor.start()` has been called for the Conductor in which the DnaInstance is running.
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
// ...
scenario.runTape("test something", (t, runner) => {
    const alice = runner.alice
    // scenario.run and scenario.runTape both inject instances
    const callResult = alice.call('people', 'create_person', {name: 'Franklin'})
})
```

> Note that there are some cases where, for the purposes of testing, you may wish to wait for the results of calling a
function in one instance, in terms of chain actions like commits and linking, to propogate to the other instances. For this,
extra ways of performing calls have been added as utilities. Check them out in [handling asynchronous network effects](./handling_async.md).

