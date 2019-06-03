const path = require('path')
const tape = require('tape')

const { Diorama, tapeExecutor, backwardCompatibilityMiddleware } = require('@holochain/diorama')

process.on('unhandledRejection', error => {
  // Will print "unhandledRejection err is not defined"
  console.error('got unhandledRejection:', error);
});

const dnaPath = path.join(__dirname, "../dist/app_spec.dna.json")
const dna = Diorama.dna(dnaPath, 'app-spec')

const diorama = new Diorama({
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

require('./test')(diorama.registerScenario)
require('./regressions')(diorama.registerScenario)

diorama.run()
