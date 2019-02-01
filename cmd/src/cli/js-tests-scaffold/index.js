// This test file uses the tape testing framework.
// To learn more, go here: https://github.com/substack/tape
const test = require('tape');

const { Config, Conductor } = require("@holochain/holochain-nodejs")

const dnaPath = "./dist/bundle.json"

// closure to keep config-only stuff out of test scope
const conductor = (() => {
    const agentAlice = Config.agent("alice")

    const dna = Config.dna(dnaPath)

    const instanceAlice = Config.instance(agentAlice, dna)

    const conductorConfig = Config.conductor([instanceAlice])
    return new Conductor(conductorConfig)
})()

// Initialize the Conductor
conductor.start()

const alice = conductor.makeCaller('alice', dnaPath)

test('description of example test', (t) => {
  // Make a call to a Zome function
  // indicating the capability and function, and passing it an input
    const addr = alice.call("my_zome", "create_my_entry", {"entry" : {"content":"sample content"}})

    const result = alice.call("my_zome", "get_my_entry", {"address": addr.Ok})

  // check for equality of the actual and expected results
  t.deepEqual(result, { Ok: { App: [ 'my_entry', '{"content":"sample content"}' ] } })

  // ends this test
  t.end()
})
