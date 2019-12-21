/// NB: The tryorama config patterns are still not quite stabilized.
/// See the tryorama README [https://github.com/holochain/tryorama]
/// for a potentially more accurate example

const path = require('path')
const tape = require('tape')

const { Orchestrator, Config, tapeExecutor, singleConductor, localOnly, combine  } = require('@holochain/tryorama')

process.on('unhandledRejection', error => {
  // Will print "unhandledRejection err is not defined"
  console.error('got unhandledRejection:', error);
});

const dnaPath = path.join(__dirname, "../dist/<<DNA_NAME>>.dna.json")

const orchestrator = new Orchestrator()

const dna = Config.dna(dnaPath, 'scaffold-test')
const conductorConfig = Config.gen({myInstanceName: dna})

orchestrator.registerScenario("description of example test", async (s, t) => {

  const {alice, bob} = await s.players({alice: conductorConfig, bob: conductorConfig})

  // Make a call to a Zome function
  // indicating the function, and passing it an input
  const addr = await alice.call("myInstanceName", "my_zome", "create_my_entry", {"entry" : {"content":"sample content"}})

  // Wait for all network activity to
  await s.consistency()

  const result = await alice.call("myInstanceName", "my_zome", "get_my_entry", {"address": addr.Ok})

  // check for equality of the actual and expected results
  t.deepEqual(result, { Ok: { App: [ 'my_entry', '{"content":"sample content"}' ] } })
})

orchestrator.run()
