
const binary = require('node-pre-gyp');
const fs = require('fs');
const path = require('path');

// deals with ensuring the correct version for the machine/node version
const binding_path = binary.find(path.resolve(path.join(__dirname, './package.json')));

const { ConfigBuilder, Container } = require(binding_path);

Container.prototype.callRaw = Container.prototype.call
Container.prototype.call = function (id, zome, trait, fn, params) {
    const stringInput = JSON.stringify(params);
    const rawResult = this.callRaw(id, zome, trait, fn, stringInput);
    let result;
    try {
        result = JSON.parse(rawResult);
    } catch (e) {
        console.log("JSON.parse failed to parse the result. The raw value is: ", rawResult);
        result = { error: "JSON.parse failed to parse the result", rawResult };
    }
    return result;
}

module.exports = {
    ConfigBuilder: new ConfigBuilder(),
    Container: Container,
};
