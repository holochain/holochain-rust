// This test file uses the tape testing framework.
// To learn more, go here: https://github.com/substack/tape
const { Config, Scenario } = require("@holochain/holochain-nodejs")
Scenario.setTape(require("tape"))

const dnaPath = "./dist/bundle.json"
const agentAlice = Config.agent("alice")
const dna = Config.dna(dnaPath)
const instanceAlice = Config.instance(agentAlice, dna)
const scenario = new Scenario([instanceAlice])

scenario.runTape("description of example test", (t, { alice }) => {
  // Make a call to a Zome function
  // indicating the function, and passing it an input
  const addr = alice.call("my_zome", "create_my_entry", {"entry" : {"content":"sample content"}})
  const result = alice.call("my_zome", "get_my_entry", {"address": addr.Ok})
  
  // check for equality of the actual and expected results
  t.deepEqual(result, { Ok: { App: [ 'my_entry', '{"content":"sample content"}' ] } })
})
