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
  default:
    throw new Error(`Unsupported network type: ${networkType}`)
}

const orchestrator = new Orchestrator({
  middleware,
  globalConfig: {
    network,
    logger: false
  }
})

require('./all-on')(orchestrator.registerScenario)

orchestrator.run()