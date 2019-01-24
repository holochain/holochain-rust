const test = require('tape');
const path = require('path');

const { Config, Container } = require('..')

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

test('can create a container two ways', t => {
    const container1 = Container.withInstances([instanceValid])
    const container2 = new Container(Config.container([instanceValid]))
    // unfortunately these objects are totally opaque so can't really test them
    t.deepEqual(container1, {})
    t.deepEqual(container2, {})
    t.end()
})

test('can start and stop a container', t => {
    const container = Container.withInstances([instanceValid])
    container.start()
    container.stop()
    t.end()
})

test('can start and stop a container via `run`', t => {
    const result = Container.run(
        [instanceValid], 
        (stop, {valorie}) => {
            t.equal(valorie.agentId.indexOf('007'), 0)
            stop()
            t.end()
        }
    ).catch(t.fail)
})

test('can pass options to `run`', t => {
    const result = Container.run(
        [instanceValid], 
        {debugLog: false},
        (stop, {valorie}) => {
            t.equal(valorie.agentId.indexOf('007'), 0)
            stop()
            t.end()
        }
    ).catch(t.fail)
})

test('container throws if it cannot start', t => {
    const result = Container.run([instanceInvalid], (stop, {ingrid}) => {
        t.fail("should have thrown exception!")
    }).catch(err => {
        t.equal(String(err).indexOf('Error: unable to start container'), 0)
        t.end()
    })
})
