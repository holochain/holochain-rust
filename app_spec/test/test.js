const path = require('path')
const { Config, Conductor, Scenario } = require('../../nodejs_conductor')
Scenario.setTape(require('tape'))

const dnaPath = path.join(__dirname, "../dist/app_spec.hcpkg")
const dna = Config.dna(dnaPath, 'app-spec')
const agentAlice = Config.agent("alice")
const agentBob = Config.agent("bob")
const agentCarol = Config.agent("carol")

const instanceAlice = Config.instance(agentAlice, dna)
const instanceBob = Config.instance(agentBob, dna)
const instanceCarol = Config.instance(agentCarol, dna)

const scenario1 = new Scenario([instanceAlice])
const scenario2 = new Scenario([instanceAlice, instanceBob])
const scenario3 = new Scenario([instanceAlice, instanceBob, instanceCarol])

scenario2.runTape('agentId', async (t, { alice, bob }) => {
  t.ok(alice.agentId)
  t.notEqual(alice.agentId, bob.agentId)
})

scenario1.runTape('show_env', async (t, { alice }) => {
  const result = alice.call("blog", "show_env", {})

  t.equal(result.Ok.dna_address, alice.dnaAddress)
  t.equal(result.Ok.dna_name, "HDK-spec-rust")
  t.equal(result.Ok.agent_address, alice.agentId)
  t.equal(result.Ok.agent_id, '{"nick":"alice","key":"' + alice.agentId + '"}')
})

scenario3.runTape('get sources', async (t, { alice, bob, carol }) => {
  const params = { content: 'whatever', in_reply_to: null }
  const address = await alice.callSync('blog', 'create_post', params).then(x => x.Ok)
  const address1 = await alice.callSync('blog', 'create_post', params).then(x => x.Ok)
  const address2 = await bob.callSync('blog', 'create_post', params).then(x => x.Ok)
  const address3 = await carol.callSync('blog', 'create_post', params).then(x => x.Ok)
  t.equal(address, address1)
  t.equal(address, address2)
  t.equal(address, address3)
  const sources1 = alice.call('blog', 'get_sources', { address }).Ok.sort()
  const sources2 = bob.call('blog', 'get_sources', { address }).Ok.sort()
  const sources3 = carol.call('blog', 'get_sources', { address }).Ok.sort()
  // NB: alice shows up twice because she published the same entry twice
  const expected = [alice.agentId, alice.agentId, bob.agentId, carol.agentId].sort()
  t.deepEqual(sources1, expected)
  t.deepEqual(sources2, expected)
  t.deepEqual(sources3, expected)
})

scenario1.runTape('call', async (t, { alice }) => {

  const num1 = 2
  const num2 = 2
  const params = { num1, num2 }
  const result = alice.call("blog", "check_sum", params)

  t.equal(result.Ok, 4)
})

scenario1.runTape('hash_post', async (t, { alice }) => {

  const params = { content: "Holo world" }
  const result = alice.call("blog", "post_address", params)

  t.equal(result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
})

scenario1.runTape('create_post', async (t, { alice }) => {

  const content = "Holo world"
  const in_reply_to = null
  const params = { content, in_reply_to }
  const result = alice.call("blog", "create_post", params)

  t.ok(result.Ok)
  t.notOk(result.Err)
  t.equal(result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
})

scenario2.runTape('delete_post', async (t, { alice, bob }) => {

  //create post
 const alice_create_post_result = await alice.callSync("blog", "create_post",
    { "content": "Posty", "in_reply_to": "" }
  )

  
  const bob_create_post_result = await alice.callSync("blog", "posts_by_agent",
    { "agent": alice.agentId }
  )

 

   t.ok(bob_create_post_result.Ok)
   t.equal(bob_create_post_result.Ok.addresses.length, 1);

  //remove link by alicce
    await alice.callSync("blog", "delete_post",
    { "content": "Posty", "in_reply_to": "" }
  )
 
  // get posts by bob
  const bob_agent_posts_expect_empty = bob.call("blog", "posts_by_agent", { "agent":alice.agentId })

  t.ok(bob_agent_posts_expect_empty.Ok)
  t.equal(bob_agent_posts_expect_empty.Ok.addresses.length, 0);
  
  })

  scenario1.runTape('delete_entry_post', async (t, { alice }) => {
    t.plan(3)
  
    const content = "Hello Holo world 321"
    const in_reply_to = null
    const params = { content, in_reply_to }
    const createResult = alice.call("blog", "create_post", params)
  
    t.ok(createResult.Ok)
  
    const deletionParams = { post_address: createResult.Ok }
    const deletionResult = alice.call("blog", "delete_entry_post", deletionParams)
  
    t.equals(deletionResult.Ok, null)
  
    const paramsGet = { post_address: createResult.Ok }
    const result = alice.call("blog", "get_post", paramsGet)
  
    t.equals(result.Ok, null)
  })

scenario1.runTape('update_post', async (t, { alice }) => {
  t.plan(4)

  const content = "Hello Holo world 123"
  const in_reply_to = null
  const params = { content, in_reply_to }
  const createResult = alice.call("blog", "create_post", params)

  t.ok(createResult.Ok)

  const updateParams = { post_address: createResult.Ok, new_content: "Hello Holo" }
  const result = alice.call("blog", "update_post", updateParams)

  t.equals(result.Ok, null)

  const updatedPost = alice.call("blog", "get_post", { post_address: createResult.Ok })

  t.ok(updatedPost.Ok)

  t.deepEqual(JSON.parse(updatedPost.Ok.App[1]), { content: "Hello Holo", date_created: "now" })
})

 scenario1.runTape('create_post with bad reply to', async (t, { alice }) => {
  t.plan(5)

  const content = "Holo world"
  const in_reply_to = "bad"
  const params = { content, in_reply_to }
  const result = alice.call("blog", "create_post", params)

  // bad in_reply_to is an error condition
  t.ok(result.Err)
  t.notOk(result.Ok)
  const error = JSON.parse(result.Err.Internal)
  t.deepEqual(error.kind, { ErrorGeneric: "Base for link not found" })
  t.ok(error.file)
  t.equal(error.line, "94")
})

scenario2.runTape('delete_post_with_bad_link', async (t, { alice, bob }) => {

  const result_bob_delete = await bob.callSync("blog", "delete_post",
    { "content": "Bad"}
  )
  
   // bad in_reply_to is an error condition
   t.ok(result_bob_delete.Err)
   t.notOk(result_bob_delete.Ok)
   const error = JSON.parse(result_bob_delete.Err.Internal)
   t.deepEqual(error.kind, { ErrorGeneric: "Target for link not found" })
   t.ok(error.file)
   t.equal(error.line, "94")
  
  })

scenario1.runTape('post max content size 280 characters', async (t, { alice }) => {

  const content = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum."
  const in_reply_to = null
  const params = { content, in_reply_to }
  const result = alice.call("blog", "create_post", params)

  // result should be an error
  t.ok(result.Err);
  t.notOk(result.Ok)

  const inner = JSON.parse(result.Err.Internal)

  t.ok(inner.file)
  t.deepEqual(inner.kind, { "ValidationFailed": "Content too long" })
  t.equals(inner.line, "94")
})

scenario1.runTape('posts_by_agent', async (t, { alice }) => {

  const agent = "Bob"
  const params = { agent }

  const result = alice.call("blog", "posts_by_agent", params)

  t.deepEqual(result.Ok, { "addresses": [] })
})

scenario1.runTape('my_posts', async (t, { alice }) => {

  await alice.callSync("blog", "create_post",
    { "content": "Holo world", "in_reply_to": "" }
  )

  await alice.callSync("blog", "create_post",
    { "content": "Another post", "in_reply_to": "" }
  )

  const result = alice.call("blog", "my_posts", {})

  t.equal(result.Ok.addresses.length, 2)
})


scenario1.runTape('my_posts_immediate_timeout', async (t, { alice }) => {

  alice.call("blog", "create_post",
    { "content": "Holo world", "in_reply_to": "" }
  )

  const result = alice.call("blog", "my_posts_immediate_timeout", {})

  t.ok(result.Err)
  console.log(result)
  t.equal(JSON.parse(result.Err.Internal).kind, "Timeout")
})

scenario1.runTape('create/get_post roundtrip', async (t, { alice }) => {

  const content = "Holo world"
  const in_reply_to = null
  const params = { content, in_reply_to }
  const create_post_result = alice.call("blog", "create_post", params)
  const post_address = create_post_result.Ok

  const params_get = { post_address }
  const result = alice.call("blog", "get_post", params_get)

  const entry_value = JSON.parse(result.Ok.App[1])
  t.comment("get_post() entry_value = " + entry_value + "")
  t.equal(entry_value.content, content)
  t.equal(entry_value.date_created, "now")

})

scenario1.runTape('get_post with non-existant address returns null', async (t, { alice }) => {

  const post_address = "RANDOM"
  const params_get = { post_address }
  const result = alice.call("blog", "get_post", params_get)

  // should be Ok value but null
  // lookup did not error
  // successfully discovered the entry does not exity
  const entry = result.Ok
  t.same(entry, null)
})

scenario2.runTape('scenario test create & publish post -> get from other instance', async (t, { alice, bob }) => {

  const initialContent = "Holo world"
  const params = { content: initialContent, in_reply_to: null }
  const create_result = await alice.callSync("blog", "create_post", params)

  const params2 = { content: "post 2", in_reply_to: null }
  const create_result2 = await bob.callSync("blog", "create_post", params2)

  t.equal(create_result.Ok.length, 46)
  t.equal(create_result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")

  const post_address = create_result.Ok
  const params_get = { post_address }

  const result = bob.call("blog", "get_post", params_get)
  const value = JSON.parse(result.Ok.App[1])
  t.equal(value.content, initialContent)
})
