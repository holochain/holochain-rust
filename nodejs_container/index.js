
const binary = require('node-pre-gyp');
const path = require('path');

// deals with ensuring the correct version for the machine/node version
const binding_path = binary.find(path.resolve(path.join(__dirname, './package.json')));

const { makeConfig, Habitat } = require(binding_path);

const promiser = (fulfill, reject) => (err, val) => {
    if (err) {
        reject(err)
    } else {
        fulfill(val)
    }
}

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

class Scenario {
    constructor(instances) {
        this.instances = instances
        this.config = Config.build(...instances)
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
        const hab = new Habitat(this.config)
        hab.start()
        const callers = {}
        this.instances.forEach(instance => {
            const id = `${instance.agent.name}-${instance.dna.path}`
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

const Config = {
    agent: name => ({ name }),
    dna: (path) => ({ path }),
    instance: (agent, dna, name) => {
        if (!name) {
            name = agent.name
        }
        return { agent, dna, name }
    },
    build: (...instances) => makeConfig(...instances),
    scenario: (...instances) => new Scenario(instances),
}

module.exports = { Config, Habitat, Scenario };
