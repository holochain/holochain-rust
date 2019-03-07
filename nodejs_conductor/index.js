const binary = require('node-pre-gyp');
const path = require('path');
const tape = require('tape');

// deals with ensuring the correct version for the machine/node version
const binding_path = binary.find(path.resolve(path.join(__dirname, './package.json')));

const { makeConfig, TestConductor: Conductor } = require(binding_path);

// Create a traditional callback function from the functions that define a Promise
const callbackFromPromise = (fulfill, reject) => (err, val) => {
    if (err) {
        reject(err)
    } else {
        fulfill(val)
    }
}

/////////////////////////////////////////////////////////////

const defaultOpts = {
    debugLog: false
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
            this._start(callbackFromPromise(fulfill, reject))
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
Conductor.run = function (config, fn) {
    const conductor = new Conductor(config)
    return new Promise((fulfill, reject) => {
        try {
            conductor._start(callbackFromPromise(fulfill, reject))
            const promise = fn(() => conductor._stop(), conductor)
            if (promise && promise.catch) {
                // If the function returned a promise, pass on its potential rejection
                // to the outer promise
                // promise.catch(reject)
            }
            // Otherwise, it should have thrown a normal Exception, which will be caught here
        } catch (e) {
            reject(e)
        }
    })
}

/////////////////////////////////////////////////////////////

class DnaInstance {
    constructor(instanceId, conductor) {
        this.id = instanceId
        this.conductor = conductor
        this.agentId = this.conductor.agent_id(instanceId)
        this.dnaAddress = this.conductor.dna_address(instanceId)
    }

    // internally calls `this.conductor._callRaw`
    call(zome, fn, params) {
        const stringInput = JSON.stringify(params)
        let rawResult
        let result
        try {
            rawResult = this.conductor._callRaw(this.id, zome, fn, stringInput)
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

    // internally calls `this.call`
    callWithPromise(...args) {
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
    callSync(...args) {
        const [result, promise] = this.callWithPromise(...args)
        return promise.then(() => result)
    }
}

/////////////////////////////////////////////////////////////

class Scenario {
    constructor(instanceConfigs, opts=defaultOpts) {
        this.instanceConfigs = instanceConfigs
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

    runTape(description, fn) {
        if (!Scenario._tape) {
            throw new Error("must call `scenario.setTape(require('tape'))` before running tape-based tests!")
        }
        Scenario._tape(description, t => {
            this.run((stop, instances) => {
                return fn(t, instances).then(() => stop())
            })
            .catch(e => t.fail(e))
            .then(t.end)
        })
    }
}

/////////////////////////////////////////////////////////////

module.exports = { Config, DnaInstance, Conductor, Scenario };
