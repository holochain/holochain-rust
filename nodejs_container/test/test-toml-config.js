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
[instances.storage]
type = "memory"

[[instances]]
id = "test/instance/2"
dna = "test/dna"
agent = "test/agent/2"
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

[logger]
type = "debug"
`

test('can create config from TOML', t => {
    const container = new Container(toml)
    container.start()
    t.throws(
        () => container.call('x', 'x', 'x', 'x'),
        /No instance with id/
    )
    t.throws(
        () => container.call(
            'test/instance/1', 'blog', 'not-a-function', 'param'
        ),
        /Zome function .*? not found/
    )
    t.end()
})
