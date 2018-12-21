const test = require('tape');

const { Container } = require('..')

const toml = `
[[agents]]
id = "test/agent/1"
name = "Holo Tester 1"
key_file = "holo_tester.key"
public_address = "sandwich--------------------------------------------------------------------------AAAEqzh28L"

[[agents]]
id = "test/agent/2"
name = "Holo Tester 2"
key_file = "holo_tester.key"
public_address = "sandwich--------------------------------------------------------------------------AAAEqzh28L"

[[dnas]]
id = "test/dna"
file = "../app_spec/dist/app_spec.hcpkg"
hash = "Qm328wyq38924y"

[[instances]]
id = "test/instance/1"
dna = "test/dna"
agent = "test/agent/1"
[instances.logger]
type = "simple"
file = "app_spec.log"
[instances.storage]
type = "memory"

[[instances]]
id = "test/instance/2"
dna = "test/dna"
agent = "test/agent/2"
[instances.logger]
type = "simple"
file = "app_spec.log"
[instances.storage]
type = "memory"

[[interfaces]]
id = "test/interface"
[interfaces.driver]
type = "websocket"
port = 8888
[[interfaces.instances]]
id = "test/instance/1"
[[interfaces.instances]]
id = "test/instance/2"
`

test('can create config from TOML', t => {
    const hab = new Container(toml)
    hab.start()
    t.throws(
        () => hab.call('x', 'x', 'x', 'x', 'x'),
        /No instance with id/
    )
    t.throws(
        () => hab.call(
            'test/instance/1', 'blog', 'main', 'not-a-function', 'param'
        ),
        /Zome function .*? not found/
    )
    t.end()
})