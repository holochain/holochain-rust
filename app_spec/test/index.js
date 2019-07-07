const path = require('path')
const tape = require('tape')

const { Orchestrator, tapeExecutor, backwardCompatibilityMiddleware } = require('@holochain/try-o-rama')
const {genConfig, spawnConductor} = require('./spawn_conductors')

process.on('unhandledRejection', error => {
  // Will print "unhandledRejection err is not defined"
  console.error('got unhandledRejection:', error);
});

const dnaPath = path.join(__dirname, "../dist/app_spec.dna.json")
const dna = Orchestrator.dna(dnaPath, 'app-spec')
const dna2 = Orchestrator.dna(dnaPath, 'app-spec', {uuid: 'altered-dna'})

const commonConductorConfig = {
  instances: {
    app: dna,
  },
}

const orchestratorSimple = new Orchestrator({
  conductors: {
    alice: commonConductorConfig,
    bob: commonConductorConfig,
    carol: commonConductorConfig,
  },
  genConfig,
  spawnConductor,
  debugLog: false,
  executor: tapeExecutor(require('tape')),
  // middleware: backwardCompatibilityMiddleware,
})

const orchestratorMultiDna = new Orchestrator({
  conductors: {
    conductor: {
      instances: {
        app1: dna,
        app2: dna2,
      },
      bridges: [
        Orchestrator.bridge('test-bridge', 'app1', 'app2')
      ],
    }
  },
  genConfig,
  spawnConductor,
  debugLog: false,
  executor: tapeExecutor(require('tape')),
  // middleware: backwardCompatibilityMiddleware,
  callbacksPort: 8888,
})

require('./regressions')(orchestratorSimple.registerScenario)
require('./test')(orchestratorSimple.registerScenario)
require('./multi-dna')(orchestratorMultiDna.registerScenario)

const run = async () => {

  await orchestratorSimple.run()

  // Multi instance tests where n3h is the network connecting them currently fails with the 2nd instance
  // waiting for and not receiving the agent entry of the first one.
  // I believe this is due to n3h not sending a peer connected message for a local instance
  // and core has not implented the authoring list yet...
  //const conductor = await spawnConductor('conductor', 6000)
  //await orchestratorMultiDna.registerConductor({name: 'conductor', url: 'http://0.0.0.0:6000'})
  //await orchestratorMultiDna.run()
  //conductor.kill()

  process.exit()
}

run()
