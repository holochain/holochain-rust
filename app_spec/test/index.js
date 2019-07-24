const path = require('path')
const tape = require('tape')

const { Orchestrator, tapeExecutor, backwardCompatibilityMiddleware } = require('@holochain/try-o-rama')
const {genConfig, spawnConductor} = require('./spawn_conductors')

// This constant serves as a check that we haven't accidentally disabled scenario tests.
// Try to keep this number as close as possible to the actual number of scenario tests.
// (But never over)
const MIN_EXPECTED_SCENARIOS = 49

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

const orchestratorValidateAgent = new Orchestrator({
  conductors: {
    valid_agent: { instances: { app: dna } },
    reject_agent: { instances: { app: dna } },
  },
  genConfig,
  spawnConductor,
  debugLog: false,
  executor: tapeExecutor(require('tape')),
  middleware: backwardCompatibilityMiddleware,
})

const registerAllScenarios = () => {
  // NB: all scenarios must be registered before any orchestrator is run. Tape will fail to register its
  // test cases if there is any Promise awaiting in between test declarations.
  let numRegistered = 0

  const registerer = orchestrator => {
    const f = (...info) => {
      numRegistered += 1
      return orchestrator.registerScenario(...info)
    }

    f.only = (...info) => {
      numRegistered += 1
      return orchestrator.registerScenario.only(...info)
    }

    return f
  }

  require('./regressions')(registerer(orchestratorSimple))
  require('./test')(registerer(orchestratorSimple))
  // require('./multi-dna')(registerer(orchestratorMultiDna))
  require('./validate-agent-test')(registerer(orchestratorValidateAgent))

  return numRegistered
}


const run = async () => {
  const num = registerAllScenarios()

  // Check to see that we haven't accidentally disabled a bunch of scenarios
  if (num < MIN_EXPECTED_SCENARIOS) {
    console.error(`Expected at least ${MIN_EXPECTED_SCENARIOS}, but only ${num} were registered!`)
    process.exit(1)
  } else {
    console.log(`Registered ${num} scenarios (at least ${MIN_EXPECTED_SCENARIOS} were expected)`)
  }

  await orchestratorSimple.run()
  // await orchestratorMultiDna.run()
  // await orchestratorValidateAgent.run()
  process.exit()
}

run()
