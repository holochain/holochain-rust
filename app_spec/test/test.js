const { Config, Container, Scenario } = require('../../nodejs_container')
Scenario.setTape(require('tape'))

const dnaPath = "./dist/app_spec.hcpkg"
const aliceName = "alice"
const tashName = "tash"

const agentAlice = Config.agent("alice")
const agentBob = Config.agent("bob")

const dna = Config.dna(dnaPath)

const instanceAlice = Config.instance(agentAlice, dna)
const instanceBob = Config.instance(agentBob, dna)

const scenario1 = new Scenario([instanceAlice])
const scenario2 = new Scenario([instanceAlice, instanceBob])

scenario2.runTape('agentId', async (t, { alice, bob }) => {
  t.plan(2)
  t.ok(alice.agentId)
  t.notEqual(alice.agentId, bob.agentId)
})

scenario1.runTape('call', async (t, { alice }) => {
  t.plan(1)

  const num1 = 2
  const num2 = 2
  const params = { num1, num2 }
  const result = alice.call("blog", "main", "check_sum", params)

  t.deepEqual(result.Ok, { "sum": "4" })
})

scenario1.runTape('hash_post', async (t, { alice }) => {
  t.plan(1)

  const params = { content: "Holo world" }
  const result = alice.call("blog", "main", "post_address", params)

  t.equal(result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
})

scenario1.runTape('create_post', async (t, { alice }) => {
  t.plan(3)

  const content = "Holo world"
  const in_reply_to = null
  const params = { content, in_reply_to }
  const result = alice.call("blog", "main", "create_post", params)

  t.ok(result.Ok)
  t.notOk(result.Err)
  t.equal(result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
})

scenario1.runTape('create_post with bad reply to', async (t, { alice }) => {
  t.plan(5)

  const content = "Holo world"
  const in_reply_to = "bad"
  const params = { content, in_reply_to }
  const result = alice.call("blog", "main", "create_post", params)

  // bad in_reply_to is an error condition
  t.ok(result.Err)
  t.notOk(result.Ok)
  const error = JSON.parse(result.Err.Internal)
  t.deepEqual(error.kind, { ErrorGeneric: "Base for link not found" })
  t.ok(error.file)
  t.equal(error.line, "86")
})

scenario1.runTape('post max content size 280 characters', async (t, { alice }) => {
  t.plan(5)

  const content = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum."
  const in_reply_to = null
  const params = { content, in_reply_to }
  const result = alice.call("blog", "main", "create_post", params)

  // result should be an error
  t.ok(result.Err);
  t.notOk(result.Ok)

  const inner = JSON.parse(result.Err.Internal)

  t.ok(inner.file)
  t.deepEqual(inner.kind, { "ValidationFailed": "Content too long" })
  t.equals(inner.line, "86")
})

scenario1.runTape('posts_by_agent', async (t, { alice }) => {
  t.plan(1)

  const agent = "Bob"
  const params = { agent }

  const result = alice.call("blog", "main", "posts_by_agent", params)

  t.deepEqual(result.Ok, { "addresses": [] })
})

scenario1.runTape('my_posts', async (t, { alice }) => {
  t.plan(1)

  await alice.callSync("blog", "main", "create_post",
    { "content": "Holo world", "in_reply_to": "" }
  )

  await alice.callSync("blog", "main", "create_post",
    { "content": "Another post", "in_reply_to": "" }
  )

  const result = alice.call("blog", "main", "my_posts", {})

  t.equal(result.Ok.addresses.length, 2)
})

scenario1.runTape('create/get_post roundtrip', async (t, { alice }) => {
  t.plan(2)

  const content = "Holo world"
  const in_reply_to = null
  const params = { content, in_reply_to }
  const create_post_result = alice.call("blog", "main", "create_post", params)
  const post_address = create_post_result.Ok

  const params_get = { post_address }
  const result = alice.call("blog", "main", "get_post", params_get)

  const entry_value = JSON.parse(result.Ok.App[1])
  t.comment("get_post() entry_value = " + entry_value + "")
  t.equal(entry_value.content, content)
  t.equal(entry_value.date_created, "now")

})

scenario1.runTape('get_post with non-existant address returns null', async (t, { alice }) => {
  t.plan(1)

  const post_address = "RANDOM"
  const params_get = { post_address }
  const result = alice.call("blog", "main", "get_post", params_get)

  // should be Ok value but null
  // lookup did not error
  // successfully discovered the entry does not exity
  const entry = result.Ok
  t.same(entry, null)
})

scenario2.runTape('scenario test create & publish post -> get from other instance', async (t, { alice, bob }) => {

  const initialContent = "Holo world"
  const params = { content: initialContent, in_reply_to: null }
  const create_result = await alice.callSync("blog", "main", "create_post", params)
  console.log("create_result: ", create_result)

  const params2 = { content: "post 2", in_reply_to: null }
  const create_result2 = await bob.callSync("blog", "main", "create_post", params2)
  console.log("create_result2: ", create_result2)

  t.equal(create_result.Ok.length, 46)
  t.equal(create_result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")

  const post_address = create_result.Ok
  const params_get = { post_address }

  const result = bob.call("blog", "main", "get_post", params_get)
  const value = JSON.parse(result.Ok.App[1])
  t.equal(value.content, initialContent)
})
