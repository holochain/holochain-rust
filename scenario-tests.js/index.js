// const tape = require('tape')

// these should be already set when the conductor starts it is the same for every test
const TEST_INTERFACE_URL = 'ws://localhost:3000'
const TEST_INTERFACE_ID = 'test-interface'

// helpers for creating a config
const Config = {
  dna: (path, id = path) => ({ path, id }),
  agent: (name, id = name) => ({ id, name }),
  instance: (agent, dna, id = `${dna.id}::${agent.id}`) => ({
    id,
    agent_id: agent.id,
    dna_id: dna.id
  })
}

/**
 * Represents a conductor process to which calls can be made via RPC
 *
 * @class      Conductor (name)
 */
class Conductor {
  constructor (config, connect) {
    this.config = config
    this.webClientConnect = connect
  }

  async connect () {
    const { call } = await this.webClientConnect(TEST_INTERFACE_URL)
    this.call = call
  }

  /**
   * Calls the conductor RPC functions to initialize it according to the config
   */
  async initialize () {
    // setup the DNAs
    // dna = {id, path}
    const call = this.call
    await this.config.dnas.forEach(async dna => {
      await call('admin/dna/install_from_file')(dna)
    })

    // setup the agents
    // agent = {id, name}
    await this.config.agents.forEach(async agent => {
      await call('admin/agent/add')({ ...agent, passphrase: '' })
    })

    // setup the instances, start them then add to the test interface
    // instance = {id, dna_id, agent_id}
    await this.config.instances.forEach(async instance => {
      await call('admin/instance/add')(instance)
      await call('admin/instance/start')(instance)
      await call('admin/interface/add_instance')({ interface_id: TEST_INTERFACE_ID, instance_id: instance.id })
    })
  }
}

/**
 * A scenario represents the execution of a number of functions against a fresh conductor specified by a given configuration
 *
 * @class      Scenario (name)
 */
class Scenario {
}

module.exports = { Config, Conductor, Scenario }
