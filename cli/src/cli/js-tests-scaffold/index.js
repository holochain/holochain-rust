

///////////////////////////////////
///
///
///            TODO
///
///
////////////////////////////////////

const path = require('path')
const tape = require('tape')

const { Orchestrator, Config, tapeExecutor } = require('@holochain/try-o-rama')

process.on('unhandledRejection', error => {
  // Will print "unhandledRejection err is not defined"
  console.error('got unhandledRejection:', error);
});

const dnaPath = path.join(__dirname, "../dist/<<DNA_NAME>>.dna.json")
const dna = Diorama.dna(dnaPath, '<<DNA_NAME>>')

const diorama = new Diorama({
  instances: {
    alice: dna,
    bob: dna,
  },
  bridges: [],
  debugLog: false,
  executor: tapeExecutor(require('tape')),
  middleware: backwardCompatibilityMiddleware,
})

diorama.registerScenario("description of example test", async (s, t, { alice }) => {
  // Make a call to a Zome function
  // indicating the function, and passing it an input
  const addr = await alice.call("my_zome", "create_my_entry", {"entry" : {"content":"sample content"}})
  const result = await alice.call("my_zome", "get_my_entry", {"address": addr.Ok})

  // check for equality of the actual and expected results
  t.deepEqual(result, { Ok: { App: [ 'my_entry', '{"content":"sample content"}' ] } })
})

diorama.run()
