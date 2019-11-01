const { expect } = require('chai')

const hc = require('./index')

describe('holochain_conductor_wasm Suite', () => {
  it('should fast_foo', () => {
    let input = 'bararas'
    let output = input
    let res = hc.fast_foo(input)
    expect(Buffer.from(res).toString()).equals(output)
  })
})
