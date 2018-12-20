
const binary = require('node-pre-gyp');
const fs = require('fs');
const path = require('path');

// deals with ensuring the correct version for the machine/node version
const binding_path = binary.find(path.resolve(path.join(__dirname, './package.json')));

const { ConfigBuilder, Habitat } = require(binding_path);

Habitat.prototype._call = Habitat.prototype.call
Habitat.prototype.call = function (id, zome, trait, fn, params, callback) {
    const stringInput = JSON.stringify(params);
    const rawResult = callback
        ? this._call(id, zome, trait, fn, stringInput, callback)
        : this._call(id, zome, trait, fn, stringInput)
    let result;
    try {
        result = JSON.parse(rawResult);
    } catch (e) {
        console.log("JSON.parse failed to parse the result. The raw value is: ", rawResult);
        result = { error: "JSON.parse failed to parse the result", rawResult };
    }
    console.log('here it comes: ', result);
    return result;
}
Habitat.prototype.callSync = function (...args) {
    // TODO: this doesn't work because the return value is not getting passed to fulfill
    // either need to pass the result through channels to get to the final callback,
    // or do this in two steps (add another neon function to register the callback)
    return new Promise((fulfill, reject) => {
        this.call(...args, (err, val) => {
            if (err) reject(err)
            else fulfill(val)
        })
    }).then(r => {
        console.log('as promised: ', r);
        return r;
    })
}

module.exports = {
    ConfigBuilder: new ConfigBuilder(),
    Habitat: Habitat,
};
