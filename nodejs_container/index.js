
const binary = require('node-pre-gyp');
const path = require('path');

// deals with ensuring the correct version for the machine/node version
const binding_path = binary.find(path.resolve(path.join(__dirname, './package.json')));

const { makeInstanceId, makeConfig, Habitat } = require(binding_path);

const promiser = (fulfill, reject) => (err, val) => {
    if (err) {
        reject(err)
    } else {
        fulfill(val)
    }
}

/////////////////////////////////////////////////////////////

const Config = {
    agent: name => ({ name }),
    dna: (path) => ({ path }),
    instance: (agent, dna, name) => {
        if (!name) {
            name = agent.name
        }
        return { agent, dna, name }
    },
    container: (...instances) => makeConfig(...instances)
}

/////////////////////////////////////////////////////////////

Habitat.prototype._start = Habitat.prototype.start
Habitat.prototype._stop = Habitat.prototype.stop
Habitat.prototype._callRaw = Habitat.prototype.call

Habitat.prototype.start = function () {
    this._stopPromise = new Promise((fulfill, reject) => {
        this._start(promiser(fulfill, reject))
    })
}

Habitat.prototype.stop = function () {
    this._stop()
    return this._stopPromise
}

Habitat.prototype._call = function (id, zome, trait, fn, params) {
    const stringInput = JSON.stringify(params);
    const rawResult = this._callRaw(id, zome, trait, fn, stringInput);
    let result;
    try {
        result = JSON.parse(rawResult);
    } catch (e) {
        console.log("JSON.parse failed to parse the result. The raw value is: ", rawResult);
        result = { error: "JSON.parse failed to parse the result", rawResult };
    }
    return result;
}

Habitat.prototype.call = function (...args) {
    this.register_callback(() => console.log("Another call well done"))
    return this._call(...args)
}

Habitat.prototype.callWithPromise = function (...args) {
    const promise = new Promise((fulfill, reject) => {
        this.register_callback(() => fulfill(result))
    })
    const result = this._call(...args)
    return [result, promise]
}

Habitat.prototype.callSync = function (...args) {
    const [result, promise] = this.callWithPromise(...args)
    return promise
        .catch(err => console.error("Error with scenario test system: ", err))
        .then(() => { return result })
}

// Convenience function for making an object that can call into the container
// in the context of a particular instance. This may be temporary.
Habitat.prototype.makeCaller = function (agentId, dnaPath) {
  const instanceId = makeInstanceId(agentId, dnaPath)
  return {
    call: (zome, cap, fn, params) => this.call(instanceId, zome, cap, fn, params),
    agentId: this.agent_id(instanceId)
  }
}

Habitat.withInstances = function (...instances) {
    const networkName = `auto-mock-network-${this._nextMock++}`
    const config = makeConfig(networkName, instances)
    return new Habitat(config)
}
// counter to give a unique mock network name for each new Habitat
Habitat._nextMock = 1

/////////////////////////////////////////////////////////////

class Scenario {
    constructor(instances) {
        this.instances = instances
    }

    static setTape(tape) {
        this.tape = tape
    }

    /**
     * Run a test case, specified by a curried function:
     * stop => (...instances) => { body }
     * where stop is a function that ends the test and shuts down the running Container
     * and the ...instances are the instances specified in the config
     * e.g.:
     *      scenario.run(stop => async (alice, bob, carol) => {
     *          const resultAlice = await alice.callSync(...)
     *          const resultBob = await bob.callSync(...)
     *          assert(resultAlice === resultBob)
     *          stop()
     *      })
     */
    run(fn) {
        const hab = Habitat.withInstances(...this.instances)
        hab.start()
        const callers = {}
        this.instances.forEach(instance => {
            const id = makeInstanceId(instance.agent.name, instance.dna.path)
            const name = instance.name
            if (name in callers) {
                throw `instance with duplicate name '${name}', please give one of these instances a new name,\ne.g. Config.instance(agent, dna, "newName")`
            }
            callers[name] = {
                call: (...args) => hab.call(id, ...args),
                callSync: (...args) => hab.callSync(id, ...args),
                callWithPromise: (...args) => hab.callWithPromise(id, ...args),
                agentId: hab.agent_id(id)
            }
        })
        fn(() => hab.stop(), callers)
    }

    runTape(tape, description, fn) {
        tape(description, t => {
            this.run(async (stop, instances) => {
                await fn(t, instances)
                t.end()
                await stop()
            })
        })
    }
}

/////////////////////////////////////////////////////////////

module.exports = { Config, Habitat, Scenario };
