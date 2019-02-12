const binary = require('node-pre-gyp');
const path = require('path');
const tape = require('tape');

// deals with ensuring the correct version for the machine/node version
const binding_path = binary.find(path.resolve(path.join(__dirname, './package.json')));

const { makeConfig, TestConductor: Conductor } = require(binding_path);

const promiser = (fulfill, reject) => (err, val) => {
    if (err) {
        reject(err)
    } else {
        fulfill(val)
    }
}

/////////////////////////////////////////////////////////////

const defaultOpts = {
    debugLog: true
}

const Config = {
    agent: name => ({ name }),
    dna: (path, name = `${path}`) => ({ path, name }),
    instance: (agent, dna, name = `${agent.name}`) => ({ agent, dna, name }),
    conductor: (instances, opts=defaultOpts) => makeConfig(instances, opts)
}

/////////////////////////////////////////////////////////////

Conductor.prototype._start = Conductor.prototype.start
Conductor.prototype._stop = Conductor.prototype.stop
Conductor.prototype._callRaw = Conductor.prototype.call

// DEPRECATED: use Conductor.run()
Conductor.prototype.start = function () {
    this._stopPromise = new Promise((fulfill, reject) => {
        try {
            this._start(promiser(fulfill, reject))
        } catch (e) {
            reject(e)
        }
    })
}

// DEPRECATED: use Conductor.run()
Conductor.prototype.stop = function () {
    this._stop()
    return this._stopPromise
}

Conductor.prototype.call = function (id, zome, fn, params) {
    const stringInput = JSON.stringify(params)
    let rawResult
    let result
    try {
        rawResult = this._callRaw(id, zome, fn, stringInput)
    } catch (e) {
        console.error("Exception occurred while calling zome function: ", e)
        throw e
    }
    try {
        result = JSON.parse(rawResult)
    } catch (e) {
        console.warn("JSON.parse failed to parse the result. The raw value is: ", rawResult)
        return rawResult
    }
    return result
}

Conductor.prototype.callWithPromise = function (...args) {
    try {
        const promise = new Promise((fulfill, reject) => {
            this.register_callback(() => fulfill())
        })
        const result = this.call(...args)
        return [result, promise]
    } catch (e) {
        return [undefined, Promise.reject(e)]
    }
}

Conductor.prototype.callSync = function (...args) {
    const [result, promise] = this.callWithPromise(...args)
    return promise.then(() => result)
}

// Convenience function for making an object that can call into the conductor
// in the context of a particular instance. This may be temporary.
Conductor.prototype.makeCaller = function (instanceId) {
  return {
    call: (zome, fn, params) => this.call(instanceId, zome, fn, params),
    agentId: this.agent_id(instanceId),
    dnaAddress: this.dna_address(instanceId),
  }
}

// DEPRECATED: use Conductor.run()
Conductor.withInstances = function (instances, opts=defaultOpts) {
    const config = Config.conductor(instances, opts)
    return new Conductor(config)
}

/**
 * Run a new Conductor, specified by a closure:
 * (stop, callers, conductor) => { (code to run) }
 * where `stop` is a function that shuts down the Conductor and must be called in the closure body
 * `opts` is an optional object of configuration
 * and the `callers` is an Object of instances specified in the config, keyed by "name"
 * (name is the optional third parameter of `Config.instance`)
 *
 * e.g.:
 *      Conductor.run([
 *          instanceAlice,
 *          instanceBob,
 *          instanceCarol,
 *      ], (stop, {alice, bob, carol}) => {
 *          const resultAlice = alice.call(...)
 *          const resultBob = bob.call(...)
 *          assert(resultAlice === resultBob)
 *          stop()
 *      })
 */
Conductor.run = function (instances, opts, fn) {
    if (typeof opts === 'function') {
        fn = opts
        opts = undefined
    }
    const conductor = Conductor.withInstances(instances, opts)
    return new Promise((fulfill, reject) => {
        try {
            conductor._start(promiser(fulfill, reject))
            const callers = {}
            instances.map(inst => {
                callers[inst.name] = conductor.makeCaller(inst.name)
            })
            fn(() => conductor._stop(), callers, conductor)
        } catch (e) {
            reject(e)
        }
    })
}

/////////////////////////////////////////////////////////////

class Scenario {
    constructor(instances, opts=defaultOpts) {
        this.instances = instances
        this.opts = opts
    }

    static setTape(tape) {
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
    run(fn) {
        return Conductor.run(this.instances, this.opts, (stop, _, conductor) => {
            const callers = {}
            this.instances.forEach(instance => {
                const name = instance.name
                if (name in callers) {
                    throw `instance with duplicate name '${name}', please give one of these instances a new name,\ne.g. Config.instance(agent, dna, "newName")`
                }
                callers[name] = {
                    call: (...args) => conductor.call(name, ...args),
                    callSync: (...args) => conductor.callSync(name, ...args),
                    callWithPromise: (...args) => conductor.callWithPromise(name, ...args),
                    agentId: conductor.agent_id(name),
                    dnaAddress: conductor.dna_address(name),
                }
            })
            return fn(stop, callers)
        })
    }

    runTape(description, fn) {
        if (!Scenario._tape) {
            throw new Error("must call `scenario.setTape(require('tape'))` before running tape-based tests!")
        }
        Scenario._tape(description, t => {
            this.run(async (stop, instances) => {
                await fn(t, instances)
                stop()
            })
            .catch(e => {
                t.fail(e)
            })
            .then(t.end)
        })
    }
}

/////////////////////////////////////////////////////////////

module.exports = { Config, Conductor, Scenario };
