const { Orchestrator, tapeExecutor, singleConductor, combine  } = require('@holochain/try-o-rama')

const { callSyncMiddleware } = require('./config')

// This constant serves as a check that we haven't accidentally disabled scenario tests.
// Try to keep this number as close as possible to the actual number of scenario tests.
// (But never over)
const MIN_EXPECTED_SCENARIOS = 19

process.on('unhandledRejection', error => {
  // Will print "unhandledRejection err is not defined"
  console.error('got unhandledRejection:', error);
});

var transport_config= 'memory';

if (process.env.APP_SPEC_TRANSPORT_TYPE =="websocket")
{
  transport_config = "websocket"
}

const orchestrator = new Orchestrator({
  middleware: combine(
    singleConductor,
    callSyncMiddleware,
    tapeExecutor(require('tape')),
  ),
  debugLog: false,
  networking: transport_config
})

require('./regressions')(orchestrator.registerScenario)
require('./files/test')(orchestrator.registerScenario)
require('./files/entry')(orchestrator.registerScenario)
require('./files/links')(orchestrator.registerScenario)
require('./files/memo')(orchestrator.registerScenario)
require('./files/crypto')(orchestrator.registerScenario)
require('./multi-dna')(orchestrator.registerScenario)
// require('./validate-agent-test')(orchestrator.registerScenario)


// Check to see that we haven't accidentally disabled a bunch of scenarios
const num = orchestrator.numRegistered()
if (num < MIN_EXPECTED_SCENARIOS) {
  console.error("Expected at least ${MIN_EXPECTED_SCENARIOS} scenarios, but only ${num} were registered!")
  process.exit(1)
} 
else {
  console.log(`Registered ${num} scenarios (at least ${MIN_EXPECTED_SCENARIOS} were expected)`)
}

orchestrator.run().then(stats => {
  console.log("All done.")
})
