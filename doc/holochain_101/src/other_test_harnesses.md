# Other Test Harnesses

Only [tape](https://github.com/substack/tape) is currently supported as a fully integrated test harness, but you can also run tests with more manual control using `scenario.run`, like so:

```javascript
// this example will also use `tape` as an illustration, but each test harness would have its own particular way
const tape = require('tape')

// Create a scenario object in the same fashion as the previous example
const scenario = new Scenario([instanceAlice, instanceBob])

// scenario.run only manages the Container for us now, but we have to manage the test itself
scenario.run((stop, {alice, bob}) => {
    tape("test something", t => {
        const result = alice.call('zome', 'capability', 'function', {params: 'go here'})
        t.equal(result, 'expected value')
        // the following two steps were not necessary when using runTape:
        t.end() // end the test
        stop() // use this injected function to stop the container
    })
})

scenario.run((stop, {alice, bob}) => {
    tape("test something else", t => {
        // write more tests in the same fashion
        t.equal(2 + 2, 4)
        t.end()
        stop() // but don't forget to stop the container when it's done!
    })
})
```

Using `run` allows you to manage the test yourself, only providing you with the basic help of starting and stopping a fresh container instance. 

The previous example used `tape` to show how it compares to using `runTape`, though you could have used any test harness, like Jest or Mocha. In fact, `runTape` simply calls `run` under the hood.
