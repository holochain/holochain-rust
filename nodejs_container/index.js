
const binary = require('node-pre-gyp');
const path = require('path');

// deals with ensuring the correct version for the machine/node version
const binding_path = binary.find(path.resolve(path.join(__dirname, './package.json')));

const { makeConfig, Habitat } = require(binding_path);

Habitat.prototype._call = Habitat.prototype.call

Habitat.prototype.call = function (id, zome, trait, fn, params) {
    const stringInput = JSON.stringify(params);
    const rawResult = this._call(id, zome, trait, fn, stringInput);
    let result;
    try {
        result = JSON.parse(rawResult);
    } catch (e) {
        console.log("JSON.parse failed to parse the result. The raw value is: ", rawResult);
        result = { error: "JSON.parse failed to parse the result", rawResult };
    }
    return result;
}

Habitat.prototype.callWithPromise = function (...args) {
    const promise = new Promise((fulfill, reject) => {
        this.register_callback(() => fulfill(result))
    }).then(() => console.log("HEY you promised!!"))
    const result = this.call(...args)
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
    run(outerFn) {
        const hab = new Habitat(this.config)
        hab.start()
        const innerFn = outerFn(() => hab.stop())
        const callers = this.instances.map(instance => {
            const id = `${instance.agent.name}-${instance.dna.path}`
            return {
                call: (...args) => hab.call(id, ...args),
                callSync: (...args) => hab.callSync(id, ...args),
                callWithPromise: (...args) => hab.callWithPromise(id, ...args),
                agentId: hab.agent_id(id)
            }
        })
        innerFn(...callers)
    }

    runTape(tape, description, outerFn) {
        tape(description, t => {
            const innerFn = outerFn(t)
            this.run(stop => async (...instances) => {
                await innerFn(...instances)
                t.end()
                stop()
            })
        })
    }
}

const Config = {
    agent: name => ({ name }),
    dna: path => ({ path }),
    instance: (agent, dna) => ({ agent, dna }),
    build: (...instances) => makeConfig(...instances),
    scenario: (...instances) => new Scenario(instances),
}

module.exports = { Config, Habitat, Scenario };
