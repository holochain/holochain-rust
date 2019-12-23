const { Orchestrator, tapeExecutor, singleConductor, localOnly, combine } = require('@holochain/tryorama')
import { networkType, configBatch } from './config'

process.on('unhandledRejection', error => {
  console.error('got unhandledRejection:', error);
});

const middleware =
  ( networkType === 'sim1h'
  ? combine(tapeExecutor(require('tape')), localOnly)

  : networkType === 'sim2h'
  ? combine(tapeExecutor(require('tape')), localOnly)

  : networkType === 'memory'
  ? combine(tapeExecutor(require('tape')), localOnly, singleConductor)

  : (() => {throw new Error(`Unsupported network type: ${networkType}`)})()
)

const orchestrator = new Orchestrator({
  middleware,
  waiter: {
    softTimeout: 10000,
    hardTimeout: 20000,
  },
})

// First two arguments are ts-node and the script name
const N = parseInt(process.argv[2], 10) || 10
const M = parseInt(process.argv[3], 10) || 1

console.log(`Running stress tests with N=${N}, M=${M} on ${networkType}`)

//require('./all-on')(orchestrator.registerScenario, N, M)
//require('./telephone-games')(orchestrator.registerScenario, N, M)

// the hammer count here is the largest number we think should be acceptable
// for ci to pass
//require('./zome-hammer')(orchestrator.registerScenario, 100)

//require('./gossip')(orchestrator.registerScenario)

require('./sharding')(orchestrator.registerScenario, configBatch, 1, 5, 5, 1)

orchestrator.run()
