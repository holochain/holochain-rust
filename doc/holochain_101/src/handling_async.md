# Handling Asynchronous Network Effects

In the previous example, we used `alice.call()` to call a zome function. This returns immediately with a value, even though the test network created by the conductor is still running, sending messages back and forth between agents for purposes of validation and replication, etc. In many test cases, you will want to wait until all of this network activity has died down to advance to the next step.

For instance, take the very common scenario as an example:

1. Alice runs a zome function which commits an entry, then adds a link to that entry
2. Bob runs a zome function which attempt to get links, which should include the link added by alice

If the test just uses `call()` to call that zome function, there is no guarantee that the entries committed by `alice.call` will be available on the DHT by the time `bob.call` is started. Therefore, two other functions are available.

**`alice.callSync`** returns a Promise instead of a simple value. The promise does not resolve until network activity has completed.
**`alice.callWithPromise`** is a slightly lower-level version of the same thing. It splits the value apart from the promise into a tuple  `[value, promise]`, so that the value can be acted on immediately and the promise waited upon separately.

```javascript
// If we make the closure `async`, we can use `await` syntax to keep things cleaner
scenario.run(async (stop, {alice, bob}) => {
    tape("test something", t => {
        // we can await on `callSync` immediately, causing it
        // to block until network activity has died down
        const result1 = await alice.callSync('zome', 'do_something_that_adds_links', {})
        // now bob can be sure he has access to the latest data
        const result2 = bob.call('zome', 'get_those_links', {})
        t.equal(result, 'expected value')
        // the following two steps were not necessary when using runTape:
        t.end() // end the test
        stop() // use this injected function to stop the conductor
    })
})
```

Even though we can't solve the eventual consistency problem in real life networks, we can solve them in tests when we have total knowledge about what each agent is doing.
