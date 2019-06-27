const path = require('path')
const tape = require('tape')

const { Diorama, tapeExecutor, backwardCompatibilityMiddleware } = require('@holochain/triorama')
const spawnConductor = require('./spawn_conductors')

process.on('unhandledRejection', error => {
  // Will print "unhandledRejection err is not defined"
  console.error('got unhandledRejection:', error);
});

const dnaPath = path.join(__dirname, "../dist/app_spec.dna.json")
const dna = Diorama.dna(dnaPath, 'app-spec')
const dna2 = Diorama.dna(dnaPath, 'app-spec', {uuid: 'altered-dna'})

const commonConductorConfig = {
  instances: {
    app: dna,
    app2: dna,
  },
  bridges: [
    Diorama.bridge('test-bridge', 'app', 'app2')
  ],
}

const dioramaSimple = new Diorama({
  conductors: {
    alice: commonConductorConfig,
    bob: commonConductorConfig,
    carol: commonConductorConfig,
  },
  debugLog: false,
  executor: tapeExecutor(require('tape')),
  middleware: backwardCompatibilityMiddleware,
})

const dioramaMultiDna = new Diorama({
  conductors: {
    conductor: {
      instances: {
        app1: dna,
        app2: dna2,
      },
      bridges: [
        Diorama.bridge('test-bridge', 'app1', 'app2')
      ],
    }
  },
  debugLog: false,
  executor: tapeExecutor(require('tape')),
  middleware: backwardCompatibilityMiddleware,
  callbacksPort: 8888,
})

require('./regressions')(dioramaSimple.registerScenario)
require('./test')(dioramaSimple.registerScenario)
require('./multi-dna')(dioramaMultiDna.registerScenario)

const run = async () => {
  await spawnConductor('alice', 3000)
  await dioramaSimple.registerConductor({name: 'alice', url: 'http://0.0.0.0:3000'})
  await spawnConductor('bob', 4000)
  await dioramaSimple.registerConductor({name: 'bob', url: 'http://0.0.0.0:4000'})
  await spawnConductor('carol', 5000)
  await dioramaSimple.registerConductor({name: 'carol', url: 'http://0.0.0.0:5000'})

  const delay = ms => new Promise(resolve => setTimeout(resolve, ms))
  console.log("Waiting for conductors to settle...")
  await delay(5000)
  console.log("Ok, starting tests!")
  
  await dioramaSimple.run()
  await dioramaMultiDna.run()
}

run()