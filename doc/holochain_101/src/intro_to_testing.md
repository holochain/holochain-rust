# Building Holochain Apps: Testing

In order to provide a familiar testing framework, a [nodejs](https://nodejs.org) version of the Holochain framework has been compiled using Rust to nodejs bindings. It is called ["holochain-nodejs"](https://www.npmjs.com/package/@holochain/holochain-nodejs) and is a publicly installable package on the NPM package manager for nodejs. It enables the execution of Holochain and DNA instances from nodejs.

At a basic level, here is how testing the Holochain DNA you are developing works:
- Use the `hc test` command to run a series of steps optimal for testing
- call a JS file containing tests
- In the JS file, import the nodejs Holochain Conductor
- load your packaged DNA into the Conductor, and otherwise configure it
- use exposed methods on the Conductor to make function calls to the DNA
- check that the results are what you expect them to be

For checking the results, a basic JavaScript test framework called [Tape](https://github.com/substack/tape) has received priority support thus far, but other test frameworks can be used.

You have the flexibility to write tests in quite a variety of ways, open to you to explore. This chapter will overview how to approach testing Holochain DNA.