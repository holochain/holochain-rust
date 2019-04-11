const test = require('tape');
const path = require('path');
const sinon = require('sinon');

const { Config, Scenario } = require('..')
Scenario.setTape(test)

const dnaValid = Config.dna(
    path.join(__dirname, "test.dna.json"),
    'dna-valid'
)
const dnaInvalid = Config.dna(
    path.join(__dirname, "nonexistent.dna.json"),
    'dna-invalid'
)

const agent = Config.agent("007")

const instanceValid = Config.instance(agent, dnaValid, 'valorie')
const instanceInvalid = Config.instance(agent, dnaInvalid, 'ingrid')

const mocktape = async (description, cb) => {
    const env = {
        pass: sinon.stub(),
        fail: sinon.spy(e => console.log('e', e, typeof e)),
        end: sinon.stub(),
    }
    try { await cb(env) } catch (e) {}
}

test("Scenario.runTape runs tests", async (t) => {
    let innerT = null

    Scenario.setTape(mocktape)
    const scenario = new Scenario([instanceValid])
    
    await scenario.runTape("mock test", async (mockT, { alice }) => {
        innerT = mockT
        mockT.pass('1')
        mockT.pass('2')
    });
    
    sinon.assert.calledTwice(innerT.pass)
    sinon.assert.notCalled(innerT.fail)
    sinon.assert.calledOnce(innerT.end)
    t.pass("Sinon assertions passed")
    t.end()
})

test("Scenario.runTape exits gracefully on failure", async (t) => {
    let innerT = null

    Scenario.setTape(mocktape)
    const scenario = new Scenario([instanceValid])
    
    await scenario.runTape("mock test", async (mockT, { alice }) => {
        innerT = mockT
        throw "thrown error should not hang test";
        t.pass('should never happen')
    });
    
    sinon.assert.notCalled(innerT.pass)
    sinon.assert.calledWith(innerT.fail, "thrown error should not hang test")
    sinon.assert.calledOnce(innerT.end)
    t.pass("Sinon assertions passed")
    t.end()
})
