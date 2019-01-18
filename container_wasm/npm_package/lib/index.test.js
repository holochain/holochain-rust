const { expect } = require('chai')

const hc = require('./index')

describe('holochain_container_wasm Suite', () => {
  it('should parse_agent_id', () => {
    const res = hc.parse_agent_id('sandwich--------------------------------------------------------------------------AAAEqzh28L')
    expect(Buffer.from(res).toString('base64')).equals(
      'sandwich++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++AAAA==')
  })
})
