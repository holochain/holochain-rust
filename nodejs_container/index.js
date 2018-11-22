
const binary = require('node-pre-gyp');
const fs = require('fs');
const path = require('path');

// deals with ensuring the correct version for the machine/node version
const binding_path = binary.find(path.resolve(path.join(__dirname,'./package.json')));

const HolochainApp = require(binding_path).HolochainApp;

module.exports = {
  loadAndInstantiate: function(fileName) {
    const content = fs.readFileSync(fileName);
    const jsonContent = JSON.parse(content);
    const jsonString = JSON.stringify(jsonContent);

    let app;
    try {
      app = new HolochainApp("bob", jsonString);
    } catch (e) {
      console.log("Unable to create Holochain instance");
      throw e;
    }
    
    /*
    Holochain ALWAYS expects and passes
    values serialized as Json. Within Holochain
    you will see this as JsonString.
    In order to avoid app developers having to
    write JSON.stringify and JSON.parse for every
    time they use app.call, we provide this convenience
    wrapper around the native `call` that comes out of
    the Holochain neon native bindings.
    */
    app._call = app.call;
    app.call = function(zome, trait, fn, params) {
      const stringInput = JSON.stringify(params);
      const rawResult = app._call(zome, trait, fn, stringInput);
      let result;
      try {
        result = JSON.parse(rawResult);
      } catch (e) {
        console.log("JSON.parse failed to parse the result. The raw value is: ", rawResult);
        result = { error: "JSON.parse failed to parse the result", rawResult };
      }
      return result;
    }

    return app;
  }
};
