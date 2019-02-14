const test = require('tape');
const path = require('path');

const { Config, Conductor } = require('..')

const dnaValid = Config.dna(
    path.join(__dirname, "bundle.json"),
    'dna-valid'
)
const dnaInvalid = Config.dna(
    path.join(__dirname, "nonexistent-file.json"),
    'dna-invalid'
)

const agent = Config.agent("007")

const configValid = Config.conductor([Config.instance(agent, dnaValid, 'valorie')])
const configInvalid = Config.conductor([Config.instance(agent, dnaInvalid, 'ingrid')])

test('can start and stop a conductor', t => {
    const conductor = new Conductor(configValid)
    conductor.start()
    conductor.stop()
    t.end()
})

test('can start and stop a conductor via `run`', t => {
    const result = Conductor.run(
        configValid,
        (stop, conductor) => {
            stop()
            t.end()
        }
    ).catch(t.fail)
})

test('conductor throws if it cannot start', t => {
    const result = Conductor.run(configInvalid, (stop) => {
        t.fail("should have thrown exception!")
    }).catch(err => {
        t.equal(String(err).indexOf('Error: unable to start conductor'), 0)
        t.end()
    })
})
