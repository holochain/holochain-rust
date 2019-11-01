# Other Test Harnesses

Only [tape](https://github.com/substack/tape) is currently supported as a fully integrated test harness, but you can also run tests with more manual control using `scenario.run`. Using `run` allows you to manage the test yourself, only providing you with the basic help of starting and stopping a fresh Conductor instance.

The example does still use `tape` to show how it compares to using `runTape`, but it could use any test harness, like Jest or Mocha. In fact, `runTape` simply calls `run` internally.

### `scenario.run(runner)` => `null`

Each invocation of `scenario.run` does the following:

1. Starts a fresh Conductor based on the configuration used to construct `scenario`
2. Injects the values needed for the test into a closure you provide

___
**Name** runner

**Type** `function`

**Description** `runner` is a closure: `(stop, runner) => { (code to run) }`.
- `stop` is a function that shuts down the Conductor and must be called in the closure body
- `runner` is an object containing an interface into each Instance specified in the config. The Instances are keyed by "name", as taken from the optional third parameter of [Config.instance](./testing_configuration.md#instances), which itself defaults to what was given in [Config.agent](./testing_configuration.md#agents).
___

#### Example
This example does also use `tape` as an illustration, but each test harness would have its own particular way
```javascript
const tape = require('tape')

// Create a scenario object in the same fashion as in other examples
const scenario = new Scenario([instanceAlice, instanceBob])

// scenario.run only manages the Conductor for us now, but we have to manage the test itself
scenario.run((stop, runner) => {
    const alice = runner.alice
    const bob = runner.bob
    tape("test something", t => {
        const result = alice.call('zome', 'function', {params: 'go here'})
        t.equal(result, 'expected value')
        // the following two steps were not necessary when using runTape:
        t.end() // end the test
        stop() // use this injected function to stop the conductor
    })
})

// This example uses destructuring to show a clean and simple way to get the Instances
scenario.run((stop, {alice, bob}) => {
    tape("test something else", t => {
        // write more tests in the same fashion
        t.equal(2 + 2, 4)
        t.end()
        stop() // but don't forget to stop the conductor when it's done!
    })
})
```
