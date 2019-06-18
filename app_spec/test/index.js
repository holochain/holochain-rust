const path = require('path')
const tape = require('tape')

const { Diorama, tapeExecutor, backwardCompatibilityMiddleware } = require('@holochain/diorama')

process.on('unhandledRejection', error => {
  // Will print "unhandledRejection err is not defined"
  console.error('got unhandledRejection:', error);
});

const dnaPath = path.join(__dirname, "../dist/app_spec.dna.json")
const dna = Diorama.dna(dnaPath, 'app-spec')

const dioramaMain = new Diorama({
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

const numInstances = 10
const manyInstances = {}
for (let i = 0; i < numInstances; i++) {
  manyInstances['Agent ' + i] = dna
}

const dioramaMany = new Diorama({
  instances: manyInstances,
  debugLog: true,
  executor: tapeExecutor(require('tape')),
})

require('./regressions')(dioramaMain.registerScenario)
require('./test')(dioramaMain.registerScenario)

require('./many-agent-tests')(dioramaMany.registerScenario)

const run = async () => {
  await dioramaMain.run()
  await dioramaMany.run()
}

run()