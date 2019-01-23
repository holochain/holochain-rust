# Building Holochain Apps: Testing

In order to provide a familiar testing framework, a nodejs version of the Holochain framework has been compiled using Rust to nodejs bindings. It is called ["holochain-nodejs"](https://www.npmjs.com/package/@holochain/holochain-nodejs) and is a publicly installable package on the NPM package manager for nodejs.

A basic test framework for nodejs called [Tape](https://github.com/substack/tape) has received priority support thus far, but other test frameworks can be used.

At a basic level, here is how testing the Holochain DNA you are developing works:
- import the nodejs Holochain Container
- load your packaged DNA into the Container, and otherwise configure it
- use an exposed method on the Container to make function calls to the DNA, and check that the results are what you expect them to be

This small chapter will overview how to approach testing Holochain DNA.

The tests can of course be called manually using nodejs, but you will find that using the convenience of the `hc test` command makes the process much smoother.

## hc test

By default, when you use `hc init` to [create a new project folder](./new_project.md), it creates a sub-directory called `test`. The files in that folder are equipped for testing your project.

Once you have a project folder initiated, you can run `hc test` to execute your tests. This combines the following steps:
  1. Packaging your files into a DNA file, located at `dist/bundle.json`. This step will fail if your packaging step fails.
  2. Installing build and testing dependencies, if they're not installed (`npm install`)
  4. Executing (with [holochain-nodejs](https://www.npmjs.com/package/@holochain/holochain-nodejs)) the test file found at `test/index.js`

`hc test` also has some configurable options.

If you want to run it without repackaging the DNA, run it with
```shell
hc test --skip-package
```

If your tests are in a different folder than `test`, run it with
```shell
hc test --dir tests
```
 where `tests` is the name of the folder.

If the file you wish to actually execute is somewhere besides `test/index.js` then run it with
```shell
hc test --testfile test/test.js
```
where `test/test.js` is the path of the file.

You have the flexibility to write tests in quite a variety of ways, open to you to explore.