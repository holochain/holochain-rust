const path = require('path')
const { Config, Conductor, Scenario } = require('../../nodejs_conductor')
Scenario.setTape(require('tape'))

const dnaPath = path.join(__dirname, "../dist/app_spec.dna.json")
const dna = Config.dna(dnaPath, 'app-spec')
const agentAlice = Config.agent("alice")
const agentBob = Config.agent("bob")
const agentCarol = Config.agent("carol")

const instanceAlice = Config.instance(agentAlice, dna)
const instanceBob = Config.instance(agentBob, dna)
const instanceCarol = Config.instance(agentCarol, dna)

const scenario1 = new Scenario([instanceAlice], { debugLog: true })
const scenario2 = new Scenario([instanceAlice, instanceBob], { debugLog: true })
const scenario3 = new Scenario([instanceAlice, instanceBob, instanceCarol], { debugLog: true })



scenario2.runTape('agentId', async (t, { alice, bob }) => {
  t.ok(alice.agentId)
  t.notEqual(alice.agentId, bob.agentId)
})

scenario1.runTape('show_env', async (t, { alice }) => {
  const result = alice.call("blog", "show_env", {})

  t.equal(result.Ok.dna_address, alice.dnaAddress)
  t.equal(result.Ok.dna_name, "HDK-spec-rust")
  t.equal(result.Ok.agent_address, alice.agentId)
  t.equal(result.Ok.agent_id, '{"nick":"alice","pub_sign_key":"' + alice.agentId + '"}')
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

scenario1.runTape('cross zome call', async (t, { alice }) => {

  const num1 = 2
  const num2 = 2
  const params = { num1, num2 }
  const result = alice.call("blog", "check_sum", params)
  t.notOk(result.Err)
  t.equal(result.Ok, 4)
})

scenario2.runTape('send', async (t, { alice, bob }) => {
  const params = { to_agent: bob.agentId, message: "ping" }
  const result = alice.call("blog", "check_send", params)

  //t.deepEqual(result.Ok, "Received : ping")
  //the line above results in `undefined`, so I switched to result to get the actual error, below:
  t.deepEqual(result, { Ok: { message: "ping" } })
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

  const bob_create_post_result = await bob.callSync("blog", "posts_by_agent",
    { "agent": alice.agentId }
  )

  t.ok(bob_create_post_result.Ok)
  t.equal(bob_create_post_result.Ok.addresses.length, 1);

  //remove link by alicce
  await alice.callSync("blog", "delete_post", { "content": "Posty", "in_reply_to": "" })

  // get posts by bob
  const bob_agent_posts_expect_empty = bob.call("blog", "posts_by_agent", { "agent": alice.agentId })

  t.ok(bob_agent_posts_expect_empty.Ok)
  t.equal(bob_agent_posts_expect_empty.Ok.addresses.length, 0);
})

scenario2.runTape('delete_entry_post', async (t, { alice, bob }) => {
  const content = "Hello Holo world 321"
  const in_reply_to = null
  const params = { content, in_reply_to }

  //commit create_post
  const createResult = await alice.callSync("blog", "create_post", params)

  t.ok(createResult.Ok)


  //delete entry post
  const deletionParams = { post_address: createResult.Ok }
  const deletionResult = await alice.callSync("blog", "delete_entry_post", deletionParams)

  t.notOk(deletionResult.Ok)


  //delete should fail
  const failedDelete = await bob.callSync("blog", "delete_entry_post", { post_address: createResult.Ok })
  t.deepEqual(failedDelete.Err,{ Internal: 'Unspecified' });

  //get initial entry
  const GetInitialParamsResult = bob.call("blog", "get_initial_post", { post_address: createResult.Ok })
  t.deepEqual(JSON.parse(GetInitialParamsResult.Ok.App[1]),{content: "Hello Holo world 321", date_created: "now" });
  
  const entryWithOptionsGet = { post_address: createResult.Ok}
  const entryWithOptionsGetResult = bob.call("blog", "get_post_with_options", entryWithOptionsGet);
  t.deepEqual(JSON.parse(entryWithOptionsGetResult.Ok.result.All.items[0].entry.App[1]),{content: "Hello Holo world 321", date_created: "now" })
})



scenario2.runTape('update_entry_validation', async (t, { alice, bob }) => {
   //update entry does not exist
   const updateParams = { post_address: "1234", new_content: "Hello Holo V2" }
   const UpdateResult = await bob.callSync("blog", "update_post", updateParams)

  t.deepEqual(UpdateResult.Err,{ Internal: 'failed to update post' });

  const content = "Hello Holo world 321"
  const in_reply_to = null
  const params = { content, in_reply_to }

  //commit create_post
  const createResult = await alice.callSync("blog", "create_post", params)

  t.ok(createResult.Ok)

  const updateParamsV2 = { post_address: createResult.Ok, new_content: "Hello Holo world 321" }
  const UpdateResultV2 = await bob.callSync("blog", "update_post", updateParamsV2)
  t.deepEqual(UpdateResultV2.Err,"Trying to modify with same data");


})

scenario2.runTape('update_post', async (t, { alice, bob }) => {
  const content = "Hello Holo world 123"
  const in_reply_to = null
  const params = { content, in_reply_to }

  //commit version 1
  const createResult = await alice.callSync("blog", "create_post", params)
  t.ok(createResult.Ok)

   //get v1
  const updatedPostV1 = alice.call("blog", "get_post", { post_address: createResult.Ok })
  const UpdatePostV1Content = { content: "Hello Holo world 123", date_created: "now" };
  t.ok(updatedPostV1.Ok)
  t.deepEqual(JSON.parse(updatedPostV1.Ok.App[1]),UpdatePostV1Content)

  //update to version 2
  const updatePostContentV2 = { content: "Hello Holo V2", date_created: "now" };
  const updateParamsV2 = { post_address: createResult.Ok, new_content: "Hello Holo V2" }
  const UpdateResultV2 = await bob.callSync("blog", "update_post", updateParamsV2)
  t.ok(UpdateResultV2.Ok)
  t.notOk(UpdateResultV2.Err)

  //get v2 using initial adderss
  const updatedPostv2Initial = alice.call("blog", "get_post", { post_address: createResult.Ok })
  t.ok(updatedPostv2Initial.Ok)
  t.notOk(updatedPostv2Initial.Err)
  t.deepEqual(JSON.parse(updatedPostv2Initial.Ok.App[1]), updatePostContentV2)

  //get v2 latest address
  const updatedPostv2Latest = alice.call("blog", "get_post", { post_address: UpdateResultV2.Ok })
  t.ok(updatedPostv2Latest.Ok)
  t.notOk(updatedPostv2Latest.Err)
  t.deepEqual(JSON.parse(updatedPostv2Latest.Ok.App[1]), updatePostContentV2)


   //get v1 using initial address
   const GetInitialPostV1Initial = alice.call("blog", "get_initial_post", { post_address: createResult.Ok })
   t.ok(GetInitialPostV1Initial.Ok)
   t.notOk(GetInitialPostV1Initial.Err)
   t.deepEqual(JSON.parse(GetInitialPostV1Initial.Ok.App[1]), { content: "Hello Holo world 123", date_created: "now" })
 
   //get v2 latest address
   const GetInitialPostV2Latest = alice.call("blog", "get_initial_post", { post_address: UpdateResultV2.Ok })
   t.ok(GetInitialPostV2Latest.Ok)
   t.notOk(GetInitialPostV2Latest.Err)
   t.deepEqual(JSON.parse(GetInitialPostV2Latest.Ok.App[1]),updatePostContentV2)

  //update to version 3
  const UpdatePostV3Content = { content: "Hello Holo V3", date_created: "now" };
  const updateParamsV3 = { post_address: createResult.Ok, new_content: "Hello Holo V3" }
  const UpdateResultV3 = await alice.callSync("blog", "update_post", updateParamsV3)
  t.ok(UpdateResultV3.Ok)
  t.notOk(UpdateResultV3.Err)

  //get v3 using initial adderss
  const updatedPostV3Initial = alice.call("blog", "get_post", { post_address: createResult.Ok })
  t.ok(updatedPostV3Initial.Ok)
  t.notOk(updatedPostV3Initial.Err)
  t.deepEqual(JSON.parse(updatedPostV3Initial.Ok.App[1]), UpdatePostV3Content)

  //get v3 using address of v2
  const updatedPostV3Latest = alice.call("blog", "get_post", { post_address: UpdateResultV2.Ok })
  t.ok(updatedPostV3Latest.Ok)
  t.notOk(updatedPostV3Latest.Err)
  t.deepEqual(JSON.parse(updatedPostV3Latest.Ok.App[1]), UpdatePostV3Content)

   //update to version 4
   const updatePostV4Content = { content: "Hello Holo V4", date_created: "now" };
   const updateParamsV4 = { post_address: createResult.Ok, new_content: "Hello Holo V4" }
   const UpdateResultV4 = await alice.callSync("blog", "update_post", updateParamsV4)
   t.notOk(UpdateResultV4.Err)
   t.ok(UpdateResultV4.Ok)
   
  //get history entry v4
   const entryHistoryV4Params = { post_address: UpdateResultV4.Ok}
   const entryHistoryV4 =  alice.call("blog", "get_history_post", entryHistoryV4Params)
   t.ok(UpdateResultV4.Ok)
   t.notOk(UpdateResultV4.Err)
   t.deepEqual(entryHistoryV4.Ok.items.length,1);
   t.deepEqual(JSON.parse(entryHistoryV4.Ok.items[0].entry.App[1]),updatePostV4Content);
   t.deepEqual(entryHistoryV4.Ok.items[0].meta.address,UpdateResultV4.Ok);
   t.deepEqual(entryHistoryV4.Ok.items[0].meta.crud_status,"live");

    //get history entry all
     const entryHistoryAllParams = { post_address: createResult.Ok}
     const entryHistoryAll = alice.call("blog", "get_history_post", entryHistoryAllParams)

     t.deepEqual(entryHistoryAll.Ok.items.length,4);
     t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[0].entry.App[1]),{ content: "Hello Holo world 123", date_created: "now" });
     t.deepEqual(entryHistoryAll.Ok.items[0].meta.address,createResult.Ok);
     t.deepEqual(entryHistoryAll.Ok.items[0].meta.crud_status,"modified");
     t.deepEqual(entryHistoryAll.Ok.crud_links[createResult.Ok],UpdateResultV2.Ok)

     t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[1].entry.App[1]),updatePostContentV2);
     t.deepEqual(entryHistoryAll.Ok.items[1].meta.address,UpdateResultV2.Ok);
     t.deepEqual(entryHistoryAll.Ok.items[1].meta.crud_status,"modified");
     t.deepEqual(entryHistoryAll.Ok.crud_links[UpdateResultV2.Ok],UpdateResultV3.Ok)

     t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[2].entry.App[1]),UpdatePostV3Content);
     t.deepEqual(entryHistoryAll.Ok.items[2].meta.address,UpdateResultV3.Ok);
     t.deepEqual(entryHistoryAll.Ok.items[2].meta.crud_status,"modified");
     t.deepEqual(entryHistoryAll.Ok.crud_links[UpdateResultV3.Ok],UpdateResultV4.Ok)

     t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[3].entry.App[1]),updatePostV4Content);
     t.deepEqual(entryHistoryAll.Ok.items[3].meta.address,UpdateResultV4.Ok);
     t.deepEqual(entryHistoryAll.Ok.items[3].meta.crud_status,"live");
     t.notOk(entryHistoryAll.Ok.crud_links[UpdateResultV4.Ok])

     const entryWithOptionsGet = { post_address: createResult.Ok}
     const entryWithOptionsGetResult = alice.call("blog", "get_post_with_options_latest", entryWithOptionsGet);

     t.deepEqual(JSON.parse(entryWithOptionsGetResult.Ok.App[1]),updatePostV4Content);  
})


scenario2.runTape('remove_update_modifed_entry', async (t, { alice, bob }) => {
  const content = "Hello Holo world 123"
  const in_reply_to = null
  const params = { content, in_reply_to }

  //commit version 1
  const createResult = await alice.callSync("blog", "create_post", params)
  t.ok(createResult.Ok)
   //get entry
  const updatedPostV1 = alice.call("blog", "get_post", { post_address: createResult.Ok })
  t.ok(updatedPostV1.Ok)
  t.deepEqual(JSON.parse(updatedPostV1.Ok.App[1]), { content: "Hello Holo world 123", date_created: "now" })

  //delete
  const removeParamsV2 = { post_address: createResult.Ok }
  const removeResultV2 = await bob.callSync("blog", "delete_entry_post", removeParamsV2)
  t.notOk(removeResultV2.Ok)

  //get v2 using initial adders
  const Postv2Initial = alice.call("blog", "get_initial_post", { post_address: createResult.Ok })
  t.ok(Postv2Initial.Ok)
  t.deepEqual(JSON.parse(Postv2Initial.Ok.App[1]), { content: "Hello Holo world 123", date_created: "now" })

  //failed delete
  const failedDelete = await alice.callSync("blog", "delete_entry_post", { post_address: createResult.Ok })
  t.deepEqual(failedDelete.Err,{ Internal: 'Unspecified' });
})

scenario1.runTape('create_post with bad reply to', async (t, { alice }) => {
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
  t.ok(error.line)
})

scenario2.runTape('delete_post_with_bad_link', async (t, { alice, bob }) => {

  const result_bob_delete = await bob.callSync("blog", "delete_post",
    { "content": "Bad" }
  )

  // bad in_reply_to is an error condition
  t.ok(result_bob_delete.Err)
  t.notOk(result_bob_delete.Ok)
  const error = JSON.parse(result_bob_delete.Err.Internal)
  t.deepEqual(error.kind, { ErrorGeneric: "Target for link not found" })
  t.ok(error.file)
  t.ok(error.line)
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
  t.ok(inner.line)
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
