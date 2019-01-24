
const binary = require('node-pre-gyp');
const path = require('path');
const tape = require('tape');

// deals with ensuring the correct version for the machine/node version
const binding_path = binary.find(path.resolve(path.join(__dirname, './package.json')));

const { makeInstanceId, makeConfig, TestContainer: Container } = require(binding_path);

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
    dna: (path, name) => {
        if (!name) {
            name = path
        }
        return { path, name }
    },
    instance: (agent, dna, name) => {
        console.log(agent, dna)
        if (!name) {
            name = makeInstanceId(agent.name, dna.name)
        }
        return { agent, dna, name }
    },
    container: (instances, opts=defaultOpts) => makeConfig(instances, opts)
}

/////////////////////////////////////////////////////////////

Container.prototype._start = Container.prototype.start
Container.prototype._stop = Container.prototype.stop
Container.prototype._callRaw = Container.prototype.call

Container.prototype.start = function () {
    this._stopPromise = new Promise((fulfill, reject) => {
        try {
            this._start(promiser(fulfill, reject))
        } catch (e) {
            reject(e)
        }
    })
}

Container.prototype.stop = function () {
    this._stop()
    return this._stopPromise
}

Container.prototype.call = function (id, zome, trait, fn, params) {
    const stringInput = JSON.stringify(params)
    let rawResult
    let result
    try {
        rawResult = this._callRaw(id, zome, trait, fn, stringInput)
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

Container.prototype.callWithPromise = function (...args) {
    try {
        const promise = new Promise((fulfill, reject) => {
            this.register_callback(() => fulfill())
        })
        const result = this.call(...args)
        return [result, promise]
    } catch (e) {
        return [
            undefined, 
            Promise.reject(e).catch(err => console.error("Error with scenario test system: ", err))
        ]
    }
}

Container.prototype.callSync = function (...args) {
    const [result, promise] = this.callWithPromise(...args)
    return promise
        .catch(err => console.error("Error with scenario test system: ", err))
        .then(() => { return result })
}

// Convenience function for making an object that can call into the container
// in the context of a particular instance. This may be temporary.
Container.prototype.makeCaller = function (agentId, dnaPath) {
  const instanceId = dnaPath ? makeInstanceId(agentId, dnaPath) : agentId
  return {
    call: (zome, cap, fn, params) => this.call(instanceId, zome, cap, fn, params),
    agentId: this.agent_id(instanceId)
  }
}

Container.withInstances = function (instances, opts=defaultOpts) {
    const config = Config.container(instances, opts)
    return new Container(config)
}

Container.run = function (instances, opts, fn) {
    if (typeof opts === 'function') {
        fn = opts
        opts = {}
    }
    const container = Container.withInstances(instances, opts)
    return new Promise((fulfill, reject) => {
        try {
            container._start(promiser(fulfill, reject))
            const callers = {}
            instances.map(inst => {
                callers[inst.name] = container.makeCaller(inst.name)
            })
            fn(() => container._stop(), callers, container)
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
     * where `stop` is a function that ends the test and shuts down the running Container
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
        return Container.run(this.instances, this.opts, (stop, _, container) => {
            const callers = {}
            this.instances.forEach(instance => {
                const id = makeInstanceId(instance.agent.name, instance.dna.name)
                const name = instance.name
                if (name in callers) {
                    throw `instance with duplicate name '${name}', please give one of these instances a new name,\ne.g. Config.instance(agent, dna, "newName")`
                }
                callers[name] = {
                    call: (...args) => container.call(id, ...args),
                    callSync: (...args) => container.callSync(id, ...args),
                    callWithPromise: (...args) => container.callWithPromise(id, ...args),
                    agentId: container.agent_id(id)
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
            this.run((stop, instances) => {
                console.log('......', 1)
                fn(t, instances)
                stop()
            })
            .catch(e => {
                console.log("whaaa", e); 
                t.fail(e)
            })
            .then(t.end)
        })
    }
}

/////////////////////////////////////////////////////////////

module.exports = { Config, Container, Scenario };
