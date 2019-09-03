const { Orchestrator, tapeExecutor, combine } = require('@holochain/try-o-rama')
// const { Orchestrator, tapeExecutor, combine } = require('quadrama')
const spawnConductor = require('./spawn_conductors')
const { callSyncMiddleware } = require('./config')

// This constant serves as a check that we haven't accidentally disabled scenario tests.
// Try to keep this number as close as possible to the actual number of scenario tests.
// (But never over)
const MIN_EXPECTED_SCENARIOS = 12

process.on('unhandledRejection', error => {
  // Will print "unhandledRejection err is not defined"
  console.error('got unhandledRejection:', error);
});

const orchestrator = new Orchestrator({
  middleware: combine(
    callSyncMiddleware,
    tapeExecutor(require('tape')),
  )
})

// require('./regressions')(orchestrator.registerScenario)
// require('./files/test')(orchestrator.registerScenario)
// require('./files/entry')(orchestrator.registerScenario)
require('./files/links')(orchestrator.registerScenario)
// require('./files/memo')(orchestrator.registerScenario)
// require('./files/crypto')(orchestrator.registerScenario)
// require('./multi-dna')(orchestrator.registerScenario)
// require('./validate-agent-test')(orchestrator.registerScenario)


// Check to see that we haven't accidentally disabled a bunch of scenarios
const num = orchestrator.numRegistered()
if (num < MIN_EXPECTED_SCENARIOS) {
  console.error(`Expected at least ${MIN_EXPECTED_SCENARIOS} scenarios, but only ${num} were registered!`)
  process.exit(1)
} else {
  console.log(`Registered ${num} scenarios (at least ${MIN_EXPECTED_SCENARIOS} were expected)`)
}

orchestrator.run().then(stats => {
  console.log("All done.")
})
