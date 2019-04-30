const tape = require('tape')

// these should be already set when the conductor is started by `hc test`
const TEST_INTERFACE_URL = 'ws://localhost:3000'
const TEST_INTERFACE_ID = 'test-interface'

/// //////////////////////////////////////////////////////////

const Config = {
  agent: id => ({ name: id, id }),
  dna: (path, id = `${path}`) => ({ path, id }),
  instance: (agent, dna, id = `${dna.id}::${agent.id}`) => ({
    id,
    agent,
    dna
  })
}

/// //////////////////////////////////////////////////////////

/**
 * Represents a conductor process to which calls can be made via RPC
 *
 * @class      Conductor (name)
 */
class Conductor {
  constructor (instances, connect) {
    this.instances = instances
    this.webClientConnect = connect
    this.agentIds = {}
    this.dnaAddresses = {}
  }

  async connect () {
    const { call } = await this.webClientConnect(TEST_INTERFACE_URL)
    this.call = call
  }

  /**
   * Calls the conductor RPC functions to initialize it according to the instances
   */
  async initialize () {
    const call = this.call
    await this.instances.forEach(async instance => {
      const installDnaResponse = await call('admin/dna/install_from_file')(instance.dna)
      const addAgentResponse = await call('test/agent/add')(instance.agent)

      await call('admin/instance/add')(instance)
      await call('admin/instance/start')(instance)
      await call('admin/interface/add_instance')({ interface_id: TEST_INTERFACE_ID, instance_id: instance.id })

      this.agentIds[instance.id] = addAgentResponse.agent_id
      this.dnaAddresses[instance.id] = installDnaResponse.dna_hash
    })
  }

  agent_id (instanceId) {
    return this.agentIds[instanceId]
  }

  dna_address (instanceId) {
    return this.dnaAddresses[instanceId]
  }

  register_callback (callback) {
    throw new Error('Not Implemented')
  }

  run (fn) {
    throw new Error('Not Implemented')
  }
}

class DnaInstance {
  constructor (instanceId, conductor) {
    this.id = instanceId
    this.conductor = conductor
    this.agentId = this.conductor.agent_id(instanceId)
    this.dnaAddress = this.conductor.dna_address(instanceId)
  }

  // internally calls `this.conductor.call`
  call (zome, fn, params) {
    try {
      const result = await this.conductor.call(this.id, zome, fn, params)
      return result
    } catch (e) {
      console.error('Exception occurred while calling zome function: ', e)
      throw e
    }
  }

  // internally calls `this.call`
  callWithPromise (...args) {
    try {
      const promise = new Promise((fulfill, reject) => {
        this.conductor.register_callback(() => fulfill())
      })
      const result = this.call(...args)
      return [result, promise]
    } catch (e) {
      return [undefined, Promise.reject(e)]
    }
  }

  // internally calls `this.callWithPromise`
  callSync (...args) {
    const [result, promise] = this.callWithPromise(...args)
    return promise.then(() => result)
  }
}

/// //////////////////////////////////////////////////////////

class Scenario {
  constructor (instanceConfigs, opts = defaultOpts) {
    this.instanceConfigs = instanceConfigs
    this.opts = opts
  }

  static setTape (tape) {
    Scenario._tape = tape
  }

  /**
     * Run a test case, specified by a closure:
     * (stop, {instances}) => { test body }
     * where `stop` is a function that ends the test and shuts down the running Conductor
     * and the `instances` is an Object of instances specified in the config, keyed by "name"
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
    const config = Config.conductor(this.instanceConfigs, this.opts)
    return Conductor.run(config, (stop, conductor) => {
      const instances = {}
      this.instanceConfigs.forEach(instanceConfig => {
        const name = instanceConfig.name
        if (name in instances) {
          throw `instance with duplicate name '${name}', please give one of these instances a new name,\ne.g. Config.instance(agent, dna, "newName")`
        }
        instances[name] = new DnaInstance(name, conductor)
      })
      return fn(stop, instances)
    })
  }

  runTape (description, fn) {
    if (!Scenario._tape) {
      throw new Error("must call `Scenario.setTape(require('tape'))` before running tape-based tests!")
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

/// //////////////////////////////////////////////////////////

module.exports = { Config, DnaInstance, Conductor, Scenario }
