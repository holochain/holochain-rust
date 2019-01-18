const binary = require('node-pre-gyp');
const fs = require('fs');
const path = require('path');

// deals with ensuring the correct version for the machine/node version
const binding_path = binary.find(path.resolve(path.join(__dirname, './package.json')));

const { makeConfig, Container } = require(binding_path);

Container.prototype.callRaw = Container.prototype.call

Container.prototype.call = function (id, zome, fn, params) {
    const stringInput = JSON.stringify(params);
    const rawResult = this.callRaw(id, zome, fn, stringInput);
    let result;
    try {
        result = JSON.parse(rawResult);
    } catch (e) {
        console.log("JSON.parse failed to parse the result. The raw value is: ", rawResult);
        result = { error: "JSON.parse failed to parse the result", rawResult };
    }
    return result;
}

// Convenience function for making an object that can call into the container
// in the context of a particular instance. This may be temporary.
Container.prototype.makeCaller = function (agentId, dnaPath) {
  const instanceId = agentId + '::' + dnaPath
  return {
    call: (zome, fn, params) => this.call(instanceId, zome, fn, params),
    agentId: this.agent_id(instanceId)
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
    container: (instances) => makeConfig(instances),
}

module.exports = { Config, Container };
