const path = require('path')
const tape = require('tape')

const { Playbook, tapeMiddleware } = require('@holochain/playbook')

process.on('unhandledRejection', error => {
  // Will print "unhandledRejection err is not defined"
  console.error('got unhandledRejection:', error);
});

const dnaPath = path.join(__dirname, "../dist/app_spec.dna.json")
const dna = Playbook.dna(dnaPath, 'app-spec')

const playbook = new Playbook({
  instances: {
    alice: dna,
    bob: dna,
    carol: dna,
  },
  bridges: [
    // TODO: hook up bridging once instantiation issue is sorted out
    // Playbook.bridge('test-bridge', 'alice', 'bob')
  ],
  debugLog: true,
  middleware: [
    tapeMiddleware(require('tape')),
  ],
})

require('./test')(playbook.registerScenario)
require('./regressions')(playbook.registerScenario)

playbook.runSuite().then(playbook.close)
