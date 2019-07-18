
const { Orchestrator, tapeExecutor, backwardCompatibilityMiddleware } = require('@holochain/try-o-rama')
const spawnConductor = require('./spawn_conductors')

module.exports = (dnaPath) => {

  const dna = Orchestrator.dna(dnaPath, 'app-spec')

  const orchestrator = new Orchestrator({
    conductors: {
      valid_agent: { instances: { app: dna } },
      reject_agent: { instances: { app: dna } },
    },
    debugLog: false,
    executor: tapeExecutor(require('tape')),
    middleware: backwardCompatibilityMiddleware,
  })

  orchestrator.registerScenario('An agent that does not pass validate_agent will not have a visible entry in the DHT', async (s, t, {valid_agent, reject_agent}) => {
    let get_self_result = await valid_agent.app.call("simple", "get_entry", {address: valid_agent.app.agentId})
    let get_other_result = await valid_agent.app.call("simple", "get_entry", {address: reject_agent.app.agentId})
    console.log("get self response", get_self_result)
    console.log("get invalid response", get_self_result)
    t.ok(get_self_result.Ok, "Should be able to retrieve own agent entry")
    t.notOk(get_other_result.Ok, "Should not be able to retrieve agent entry for invalid agent")
  })

  const run = async () => {
    const valid_agent = await spawnConductor('valid_agent', 3000)
    await orchestrator.registerConductor({name: 'valid_agent', url: 'http://0.0.0.0:3000'})
    const reject_agent = await spawnConductor('reject_agent', 4000)
    await orchestrator.registerConductor({name: 'reject_agent', url: 'http://0.0.0.0:4000'})

    const delay = ms => new Promise(resolve => setTimeout(resolve, ms))
    console.log("Waiting for conductors to settle...")
    await delay(5000)
    console.log("Ok, starting tests!")

    await orchestrator.run()
    valid_agent.kill()
    reject_agent.kill()

    process.exit()
  }

  run()

}
