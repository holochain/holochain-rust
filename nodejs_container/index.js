
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

const scenario = (instances, test) => {
    const hab = new Habitat(config)
}

const Config = {
    agent: name => [name],
    dna: path => [path],
    instance: (agent, dna) => ({ agent, dna }),
    build: (...args) => makeConfig(...args),
}

module.exports = { Config, Habitat };
