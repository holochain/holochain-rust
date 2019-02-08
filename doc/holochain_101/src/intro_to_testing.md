# Building Holochain Apps: Testing

In order to provide a familiar testing framework, a nodejs version of the Holochain framework has been compiled using Rust to nodejs bindings. It is called ["holochain-nodejs"](https://www.npmjs.com/package/@holochain/holochain-nodejs) and is a publicly installable package on the NPM package manager for nodejs.

A basic test framework for nodejs called [Tape](https://github.com/substack/tape) has received priority support thus far, but other test frameworks can be used.

At a basic level, here is how testing the Holochain DNA you are developing works:
- import the nodejs Holochain Conductor
- load your packaged DNA into the Conductor, and otherwise configure it
- use exposed methods on the Conductor to make function calls to the DNA
- check that the results are what you expect them to be

This small chapter will overview how to approach testing Holochain DNA.