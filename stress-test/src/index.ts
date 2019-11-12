const { Orchestrator, tapeExecutor, singleConductor, combine } = require('@holochain/try-o-rama')

process.on('unhandledRejection', error => {
  console.error('got unhandledRejection:', error);
});

const networkType = process.env.APP_SPEC_NETWORK_TYPE || 'sim1h'
let network = null
let middleware = null

switch (networkType) {
  case 'memory':
    network = 'memory'
    middleware = combine(singleConductor, tapeExecutor(require('tape')))
    break
  case 'sim1h':
    network = {
      type: 'sim1h',
      dynamo_url: "http://localhost:8000",
    }
    middleware = tapeExecutor(require('tape'))
    break
  case 'sim2h':
    network = {
      type: 'sim2h',
      sim2h_url: "wss://localhost:9002",
    }
    middleware = tapeExecutor(require('tape'))
    break
  default:
    throw new Error(`Unsupported network type: ${networkType}`)
}

const orchestrator = new Orchestrator({
  middleware,
  globalConfig: {
    network,
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
      state_dump: false
    }
  }
})

// First two arguments are ts-node and the script name
const N = parseInt(process.argv[2], 10) || 10
const M = parseInt(process.argv[3], 10) || 1

console.log(`Running stress tests with N=${N}, M=${M}`)

require('./all-on')(orchestrator.registerScenario, N, M)
require('./telephone-games')(orchestrator.registerScenario, N, M)

// the hammer count here is the largest number we think should be acceptable
// for ci to pass
require('./zome-hammer')(orchestrator.registerScenario, 100)


orchestrator.run()
