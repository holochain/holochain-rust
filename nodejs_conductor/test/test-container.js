const test = require('tape');
const path = require('path');

const { Config, Conductor } = require('..')

const dnaValid = Config.dna(
    path.join(__dirname, "../../app_spec/dist/app_spec.hcpkg"),
    'dna-valid'
)
const dnaInvalid = Config.dna(
    path.join(__dirname, "nonexistent-file.json"),
    'dna-invalid'
)

const agent = Config.agent("007")

const instanceValid = Config.instance(agent, dnaValid, 'valorie')
const instanceInvalid = Config.instance(agent, dnaInvalid, 'ingrid')

test('can create a conductor two ways', t => {
    const conductor1 = Conductor.withInstances([instanceValid])
    const conductor2 = new Conductor(Config.conductor([instanceValid]))
    // unfortunately these objects are totally opaque so can't really test them
    t.deepEqual(conductor1, {})
    t.deepEqual(conductor2, {})
    t.end()
})

test('can start and stop a conductor', t => {
    const conductor = Conductor.withInstances([instanceValid])
    conductor.start()
    conductor.stop()
    t.end()
})

test('can start and stop a conductor via `run`', t => {
    const result = Conductor.run(
        [instanceValid],
        (stop, {valorie}) => {
            t.equal(valorie.agentId.indexOf('007'), 0)
            stop()
            t.end()
        }
    ).catch(t.fail)
})

test('can pass options to `run`', t => {
    const result = Conductor.run(
        [instanceValid],
        {debugLog: false},
        (stop, {valorie}) => {
            t.equal(valorie.agentId.indexOf('007'), 0)
            stop()
            t.end()
        }
    ).catch(t.fail)
})

test('conductor throws if it cannot start', t => {
    const result = Conductor.run([instanceInvalid], (stop, {ingrid}) => {
        t.fail("should have thrown exception!")
    }).catch(err => {
        t.equal(String(err).indexOf('Error: unable to start conductor'), 0)
        t.end()
    })
})
