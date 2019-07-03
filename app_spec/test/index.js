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

const dioramaSimple = new Diorama({
  instances: {
    alice: dna,
    bob: dna,
    carol: dna,
  },
  bridges: [
    Diorama.bridge('test-bridge', 'alice', 'bob')
  ],
  debugLog: false,
  executor: tapeExecutor(require('tape')),
  middleware: backwardCompatibilityMiddleware,
})

const dioramaMultiDna = new Diorama({
  instances: {
    alice: dna,
    bob: dna2,
  },
  bridges: [
    Diorama.bridge('test-bridge', 'alice', 'bob')
  ],
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