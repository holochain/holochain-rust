// const tape = require('tape')
const { connect } = require('@holochain/hc-web-client')

const TEST_INTERFACE_URL = 'ws://localhost:3000'
const TEST_INTERFACE_ID = 'test-interface'

// helpers for creating a config
const Config = {
  dna: (path, id = path) => ({ path, id }),
  agent: (name, id = name) => {
    // create the keystore and public address

    return {
      id,
      name,
      public_address: '',
      keystore_file: ''
    }
  },
  instance: (agent, dna, name = `${agent.name}`) => ({ agent, dna, name })
}

/**
 * Represents a conductor process to which calls can be made via RPC
 *
 * @class      Conductor (name)
 */
class Conductor {
  constructor (config) {
    this.config = config
  }

  connect () {
    const { call, callZome } = connect(TEST_INTERFACE_URL)
    this.call = call
    this.callZome = callZome
  }

  /**
   * Calls the conductor RPC functions to initialize it according to the config
   */
  initialize () {
    // setup the DNAs
    // dna = {id, path}
    this.config.dnas.forEach(dna => {
      this.call('admin/dna/install_from_file')(dna)
    })

    // setup the agents
    // agent = {id, name, public_address, keystore_file}
    this.config.agents.forEach(agent => {
      this.call('admin/agent/add')(agent)
    })

    // setup the instances, start them then add to the test interface
    // instance = {id, dna_id, agent_id}
    this.config.instances.forEach(instance => {
      this.call('admin/instance/add')(instance)
      this.call('admin/instance/start')(instance)
      this.call('admin/interface/add_instance')({ interface_id: TEST_INTERFACE_ID, instance_id: instance.id })
    })
  }

  /**
   * Uses the signals to ensure all activity has stopped before progressing
   */
  callSync (instanceId, zome, func, args) {

  }
}

/**
 * An instance is an agent running a particular DNA
 *
 * @class      Instance (name)
 */
class Instance {
  // agentdId
  // dnaAddress

  // constructor (dna, agent) {

  // }

  /**
   * Call a function on a given instance
   */
  call (zome, func, args) {

  }

  /**
   * Uses the signals to ensure all activity has stopped before progressing
   */
  callSync (zome, func, args) {

  }
}

/**
 * A scenario represents the execution of a number of functions against a fresh conductor specified by a given configuration
 *
 * @class      Scenario (name)
 */
class Scenario {
  // config
  // opts

  constructor (config, opts) {
    this.config = config
    this.opts = opts
  }

  static setTape (tape) {
    Scenario._tape = tape
  }

  /**
   * Run a test case, specified by a closure:
   * (stop, {instances}) => { test body }
   * where `stop` is a function that ends the test and shuts down the running Conductor
   * and the `instances` is an Object of instances specified in the config, keyed by 'name'
   * (name is the optional third parameter of `Config.instance`)
   *
   * e.g.:
   *      scenario.run(async (stop, {alice, bob, carol}) => {
   *          const resultAlice = await alice.callSync(...)
   *          const resultBob = await bob.callSync(...)
   *          assert(resultAlice === resultBob)
   *          stop()
   *      })
   */
  run (fn) {
    const conductor = new Conductor(this.config)

    return conductor.run((stop, conductor) => {
      const instances = {}
      this.instanceConfigs.forEach(instanceConfig => {
        const name = instanceConfig.name
        if (name in instances) {
          throw new Error(`instance with duplicate name '${name}', please give one of these instances a new name,\ne.g. Config.instance(agent, dna, 'newName')`)
        }
        // instances[name] = new DnaInstance(name, conductor)
      })
      return fn(stop, instances)
    })
  }

  runTape (description, fn) {
    if (!Scenario._tape) {
      throw new Error('must call `Scenario.setTape(require(\'tape\'))` before running tape-based tests!')
    }
    return new Promise(resolve => {
      Scenario._tape(description, async t => {
        try {
          await this.run((stop, instances) => (
            fn(t, instances).then(() => stop())
          ))
        } catch (e) {
          t.fail(e)
        } finally {
          t.end()
          resolve()
        }
      })
    })
  }
}

module.exports = { Config, Instance, Conductor, Scenario }
