# Intro to holochain-nodejs

The purpose of the [holochain-nodejs](https://www.npmjs.com/package/@holochain/holochain-nodejs) module is to make integration tests and scenario tests able to be written simply and with as little boilerplate as possible. However, the module also provides even more basic functionality, making it possible to build tests with whatever tradeoff between convenience and customization is right for your project.

There are two primary capabilities of the module, which are introduced below.

## Simple, Single Node Integration Tests

The point of this mode of testing is simply to call Zome functions, and ensure that they produce the result you expect. It is discussed further in [calling zome functions](./nodejs_calling_zome_functions.md) and [checking results](./testing_checking_results.md).

## Scenario Tests

The point of this mode of testing is to launch multiple instances, call functions in one, then another, and to ensure that processes involving multiple agents play out as intended. The module conveniently provides a way to sandbox the execution of these scenarios as well, so that you can test multiple without worrying about side effects. This is discussed further in [scenario testing](./scenario_testing.md).