const path = require('path')
const tape = require('tape')

const { Diorama, tapeExecutor, backwardCompatibilityMiddleware } = require('@holochain/diorama')

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
})

require('./regressions')(dioramaSimple.registerScenario)
require('./test')(dioramaSimple.registerScenario)
require('./multi-dna')(dioramaMultiDna.registerScenario)

const run = async () => {
  await dioramaSimple.run()
  await dioramaMultiDna.run()
}

run()