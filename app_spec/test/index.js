const { Orchestrator, tapeExecutor, singleConductor, localOnly, combine, callSync  } = require('@holochain/tryorama')

// This constant serves as a check that we haven't accidentally disabled scenario tests.
// Try to keep this number as close as possible to the actual number of scenario tests.
// (But never over)
const MIN_EXPECTED_SCENARIOS = 32

process.on('unhandledRejection', error => {
  console.error('got unhandledRejection:', error);
});


const networkType = process.env.APP_SPEC_NETWORK_TYPE
const middleware =
  ( networkType === 'websocket'
  ? combine(tapeExecutor(require('tape')), localOnly, callSync)

  : networkType === 'sim2h'
  ? combine(tapeExecutor(require('tape')), localOnly, callSync)

  : networkType === 'memory'
  ? combine(tapeExecutor(require('tape')), localOnly, singleConductor, callSync)

  : (() => {throw new Error(`Unsupported memory type: ${networkType}`)})()
)

const orchestrator = new Orchestrator({
  middleware,
  waiter: {
    softTimeout: 10000,
    hardTimeout: 20000,
  }
})

require('./regressions')(orchestrator.registerScenario)
require('./files/test')(orchestrator.registerScenario)
require('./files/entry')(orchestrator.registerScenario)
require('./files/links')(orchestrator.registerScenario)
require('./files/memo')(orchestrator.registerScenario)
require('./files/crypto')(orchestrator.registerScenario)
require('./files/offline-validation')(orchestrator.registerScenario)
require('./multi-dna')(orchestrator.registerScenario)
require('./offline')(orchestrator.registerScenario)
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
  console.log('All done.')
})
