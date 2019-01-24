const test = require('tape');
const path = require('path');

const { Config, Scenario } = require('..')

const dnaValid = Config.dna(
    path.join(__dirname, "../../app_spec/dist/app_spec.hcpkg"), 
    'dna-valid'
)
const dnaInvalid = Config.dna(
    path.join(__dirname, "nonexistent-file.json"), 
    'dna-invalid'
)

const agent = Config.agent("007")

const nameValid = '007::dna-valid'
const nameInvalid = '007::dna-invalid'
// const instanceValid = Config.instance(agent, dnaValid, 'valorie')
// const instanceInvalid = Config.instance(agent, dnaInvalid, 'ingrid')
const instanceValid = Config.instance(agent, dnaValid, nameValid)
const instanceInvalid = Config.instance(agent, dnaInvalid, nameInvalid)

test('can run a scenario', t => {
    const scenario = new Scenario([instanceValid])
    scenario.run((stop, peeps) => {
        t.equal(peeps[nameValid].agentId[2], '7')
        t.end()
        stop()
    }).catch(t.fail)
})

test('scenario throws if container cannot start', t => {
    const scenario = new Scenario([instanceInvalid])
    scenario.run((stop, peeps) => {
        t.fail('should have thrown exception')
    }).catch(err => {
        t.equal(String(err).indexOf('Error: unable to start container'), 0)
        t.end()
    })
})
