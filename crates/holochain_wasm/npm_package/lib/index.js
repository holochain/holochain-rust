const hcTargetPrefix = process.env.HC_TARGET_PREFIX;
module.exports = require(hcTargetPrefix + '/crates/holochain_wasm/npm_package/gen')
