## Run Tests With Tape

### `scenario.runTape(description, runner)` => `null`

Each invocation of `scenario.runTape` does the following:

1. Starts a fresh Conductor based on the configuration used to construct `scenario`
2. Starts a new `tape` test
3. Injects the values needed for the test into a closure you provide
4. Automatically ends the test and stops the conductor when the closure is done running

It will error if you have not called [Scenario.setTape](./scenario_testing_setup.md#scenariosettapetape--null) first.

___
**Name** description

**Type** `string`

**Description** Will be used to initialize the tape test, and should describe that which is being tested in this scenario
___
**Name** runner

**Type** `function`

**Description** `runner` is a closure: `(t, runner) => { (code to run) }`. When this function ends, the test is automatically ended, and the inner Conductor is stopped.
- `t` is the object that tape tests use
- `runner` is an object containing an interface into each Instance specified in the config. The Instances are keyed by "name", as taken from the optional third parameter of [Config.instance](./testing_configuration.md#instances), which itself defaults to what was given in [Config.agent](./testing_configuration.md#agents).
___

#### Example
```javascript
scenario.runTape("test something", (t, runner) => {
    const alice = runner.alice
    const bob = runner.bob
    // fire zome function calls from both agents
    const result1 = alice.call('zome', 'function', {params: 'go here'})
    const result2 = bob.call('zome', 'function', {params: 'go here'})
    // make some tape assertions
    t.ok(result1)
    t.equal(result2, 'expected value')
})

// Run another test in a freshly created Conductor
// This example uses destructuring to show a clean and simple way to get the Instances
scenario.runTape("test something else", (t, {alice, bob}) => {
    // write more tests in the same fashion
})
```


