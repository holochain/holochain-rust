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

// Conductor.prototype._start = Conductor.prototype.start
// Conductor.prototype._stop = Conductor.prototype.stop
// Conductor.prototype._callRaw = Conductor.prototype.call

// // DEPRECATED: use Conductor.run()
// Conductor.prototype.start = function () {
//     this._stopPromise = new Promise((fulfill, reject) => {
//         try {
//             this._start(callbackFromPromise(fulfill, reject))
//         } catch (e) {
//             reject(e)
//         }
//     })
// }

// // DEPRECATED: use Conductor.run()
// Conductor.prototype.stop = function () {
//     this._stop()
//     return this._stopPromise
// }

/**
 * Run a new Conductor, specified by a closure which returns a Promise:
 * (stop, conductor) => { (code to run) }
 * where `stop` is a function that shuts down the Conductor and must be called in the closure body
 *
 * e.g.:
 *      Conductor.run(Config.conductor([
 *          instanceAlice,
 *          instanceBob,
 *          instanceCarol,
 *      ]), (stop, conductor) => {
 *          doStuffWith(conductor)
 *          stop()
 *      })
 */
// Conductor.run = function (config, fn) {
//     const conductor = new Conductor(config)
//     return new Promise(async (fulfill, reject) => {
//         try {
//             conductor._start(callbackFromPromise(fulfill, reject))
//             await fn(() => conductor._stop(), conductor)
//         } catch (e) {
//             conductor._stop()
//             reject(e)
//         }
//     })
// }

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
      const dnaAddress = await call('admin/dna/install_from_file')(instance.dna)
      const agentId = await call('test/agent/add')(instance.agent)

      await call('admin/instance/add')(instance)
      await call('admin/instance/start')(instance)
      await call('admin/interface/add_instance')({ interface_id: TEST_INTERFACE_ID, instance_id: instance.id })

      this.agentIds[instance.id] = agentId
      this.dnaAddresses[instance.id] = dnaAddress
    })
  }

  agent_id (instanceId) {
    throw new Error('Not Implemented')
  }

  dna_address (instanceId) {
    throw new Error('Not Implemented')
  }

  async _callRaw (nstanceId, zome, fn, stringInput) {
    throw new Error('Not Implemented')
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

  // internally calls `this.conductor._callRaw`
  call (zome, fn, params) {
    const stringInput = JSON.stringify(params)
    let rawResult
    let result
    try {
      rawResult = this.conductor._callRaw(this.id, zome, fn, stringInput)
    } catch (e) {
      console.error('Exception occurred while calling zome function: ', e)
      throw e
    }
    try {
      result = JSON.parse(rawResult)
    } catch (e) {
      console.warn('JSON.parse failed to parse the result. The raw value is: ', rawResult)
      return rawResult
    }
    return result
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
