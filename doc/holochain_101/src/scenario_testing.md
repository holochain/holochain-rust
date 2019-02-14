# Scenario Testing

The following example shows the simplest, most convenient way to start writing scenario tests with this module. We'll set up an environment for running tests against two instances of one DNA, using the [tape](https://github.com/substack/tape) test harness:

```javascript
const { Config, Scenario } = require('@holochain/holochain-nodejs')

Scenario.setTape(require('tape'))

// specify two agents...
const agentAlice = Config.agent("alice")
const agentBob = Config.agent("bob")
// ...and one DNA...
const dna = Config.dna("path/to/happ.hcpkg")
// ...then make instances out of them...
const instanceAlice = Config.instance(agentAlice, dna)
const instanceBob = Config.instance(agentBob, dna)

// Now we can construct a `scenario` object which lets us run as many scenario tests as we want involving the two instances we set up:
const scenario = new Scenario([instanceAlice, instanceBob])

// Run a test in a freshly created conductor. Note the various parameters used:
// - a description which will be used to initialize the tape test
// - a closure that takes two arguments which will be injected:
//   - `t` is the just the object that tape tests use
//   - the second argument is an object containing an interface into each instance specified in the config
scenario.runTape("test something", (t, {alice, bob}) => {
    // fire zome function calls from both agents
    const result1 = alice.call('zome', 'function', {params: 'go here'})
    const result2 = bob.call('zome', 'function', {params: 'go here'})
    // make some tape assertions
    t.ok(result1)
    t.equal(result2, 'expected value')

    // when this function ends, the test is automaticaly ended,
    // and the conductor is stopped
})

// Run another test in a freshly created conductor
scenario.runTape("test something else", (t, {alice, bob}) => {
    // write more tests in the same fashion
})
```

Note that we used two objects here:

* `Config`, which was used to build up a valid configuration for the scenario tests
* `Scenario`, which does all the work of starting and stopping conductors and integrating with various test harnesses (currently only `tape` is supported).

Each invocation of `scenario.runTape` does the following:

1. Starts a fresh Conductor based on the configuration used to construct `scenario`
2. Starts a new `tape` test
3. Injects the values needed for the test into a closure you provide:
    * `t`, which has the usual `tape` interface for assertions, etc.
    * an object which contains an interface to each of the instances specified by the config
4. Automatically ends the test and stops the conductor when the closure is done running
