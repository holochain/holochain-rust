const test = require('tape');
const path = require('path');

const { Config, Scenario } = require('..')

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

test('can run a scenario', t => {
    const scenario = new Scenario([instanceValid])
    scenario.run((stop, {valorie}) => {
        t.equal(valorie.agentId, "HcSCjaqeDMMwaa3evjVVzb3RcCH8s5apz6habwxAwIcs37mz9D6QXdIWY499Yja")
        t.end()
        stop()
    }).catch(t.fail)
})

test('can run a scenario (with async function)', t => {
    const scenario = new Scenario([instanceValid])
    scenario.run(async (stop, {valorie}) => {
        t.equal(valorie.agentId, "HcSCjaqeDMMwaa3evjVVzb3RcCH8s5apz6habwxAwIcs37mz9D6QXdIWY499Yja")
        t.end()
        stop()
    }).catch(t.fail)
})

test('scenario throws if conductor cannot start', t => {
    const scenario = new Scenario([instanceInvalid])
    scenario.run((stop, {ingrid}) => {
        t.fail('should have thrown exception')
    }).catch(err => {
        t.equal(String(err).indexOf('Error: unable to start conductor'), 0)
        t.end()
    })
})

test('scenario throws if conductor cannot start (with async function)', t => {
    const scenario = new Scenario([instanceInvalid])
    scenario.run(async (stop, {ingrid}) => {
        t.fail('should have thrown exception')
    }).catch(err => {
        t.equal(String(err).indexOf('Error: unable to start conductor'), 0)
        t.end()
    })
})
