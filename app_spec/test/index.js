const { Orchestrator, tapeExecutor, singleConductor, combine } = require('@holochain/try-o-rama')

const { callSyncMiddleware } = require('./config')

// This constant serves as a check that we haven't accidentally disabled scenario tests.
// Try to keep this number as close as possible to the actual number of scenario tests.
// (But never over)
const MIN_EXPECTED_SCENARIOS = 1

process.on('unhandledRejection', error => {
  // Will print "unhandledRejection err is not defined"
  console.error('got unhandledRejection:', error)
})

const dumbWaiter = interval => (run, f) => run(s =>
  f(Object.assign({}, s, {
    consistency: () => new Promise(resolve => {
      console.log(`dumbWaiter is waiting ${interval}ms...`)
      setTimeout(resolve, interval)
    })
  }))
)

let transport_config = 'memory'
let middleware = combine(
  // by default, combine conductors into a single conductor for in-memory networking
  // NB: this middleware makes a really huge difference! and it's not very well tested,
  // as of Oct 1 2019. So, keep an eye out.
  // singleConductor,
  callSyncMiddleware,
  tapeExecutor(require('tape'))
)

if (process.env.APP_SPEC_NETWORK_TYPE === 'websocket') {
  transport_config = 'websocket'

  // omit singleConductor
  middleware = combine(
    callSyncMiddleware,
    tapeExecutor(require('tape'))
  )
}

if (process.env.APP_SPEC_NETWORK_TYPE === 'sim1h') {
  transport_config = {
    type: 'sim1h',
    dynamo_url: 'http://localhost:8000'
  }

  // omit singleConductor
  middleware = combine(
    // dumbWaiter(1000),
    callSyncMiddleware,
    tapeExecutor(require('tape'))
  )
}

if (process.env.APP_SPEC_NETWORK_TYPE === 'sim2h') {
  transport_config = {
    type: 'sim2h',
    sim2h_url: 'wss://localhost:9000'
  }

  // omit singleConductor
  middleware = combine(
    // dumbWaiter(1000),
    callSyncMiddleware,
    tapeExecutor(require('tape'))
  )
}

const orchestrator = new Orchestrator({
  middleware,
  waiter: {
    softTimeout: 5000,
    hardTimeout: 10000
  },
  globalConfig: {
<<<<<<< HEAD
      logger: {
          type: "debug",
          rules: {
              rules: [
                  {
                      exclude: true,
                      pattern: ".*parity.*"
                  },
                  {
                      exclude: true,
                      pattern: ".*mio.*"
                  },
                  {
                      exclude: true,
                      pattern: ".*tokio.*"
                  },
                  {
                      exclude: true,
                      pattern: ".*hyper.*"
                  },
                  {
                      exclude: true,
                      pattern: ".*rusoto_core.*"
                  },
                  {
                      exclude: true,
                      pattern: ".*want.*"
                  },
                  {
                      exclude: true,
                      pattern: ".*rpc.*"
                  },
                  {
                      exclude: true,
                      pattern: ".*lib3h.*"
                  }
              ]
=======
    logger: {
      type: 'debug',
      rules: {
        rules: [
          {
            exclude: true,
            pattern: '.*parity.*'
          },
          {
            exclude: true,
            pattern: '.*mio.*'
          },
          {
            exclude: true,
            pattern: '.*tokio.*'
          },
          {
            exclude: true,
            pattern: '.*hyper.*'
>>>>>>> ac2b12b806f5730933a516ef805789affe22407d
          },
          {
            exclude: true,
            pattern: '.*rusoto_core.*'
          },
          {
            exclude: true,
            pattern: '.*want.*'
          },
          {
            exclude: true,
            pattern: '.*rpc.*'
          }
        ]
      },
      state_dump: true
    },
    network: transport_config
  }
})

// require('./regressions')(orchestrator.registerScenario)
// require('./files/test')(orchestrator.registerScenario)
// require('./files/entry')(orchestrator.registerScenario)
// require('./files/links')(orchestrator.registerScenario)
// require('./files/memo')(orchestrator.registerScenario)
// require('./files/crypto')(orchestrator.registerScenario)
require('./files/offline-validation')(orchestrator.registerScenario)
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
  console.log('All done.')
})
