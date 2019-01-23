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

const agent = Config.agent("ben")

const instanceValid = Config.instance(agent, dnaValid)
const instanceInvalid = Config.instance(agent, dnaInvalid)

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
    const container = Container.withInstances([instanceValid])
    container.run(stop => {
        stop()
        t.end()
    }).catch(t.fail)
})
