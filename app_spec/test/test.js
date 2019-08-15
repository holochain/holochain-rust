module.exports = scenario => {

scenario('capabilities grant and claim', async (s, t, { alice, bob }) => {

    // Ask for alice to grant a token for bob  (it's hard-coded for bob in re function for now)
    const result = await alice.app.call("blog", "request_post_grant", {})
    t.ok(result.Ok)
    t.notOk(result.Err)

    // Confirm that we can get back the grant
    const grants = await alice.app.call("blog", "get_grants", {})
    t.ok(grants.Ok)
    t.notOk(grants.Err)
    t.equal(result.Ok, grants.Ok[0])

    // Bob stores the grant as a claim
    const claim = await bob.app.call("blog", "commit_post_claim", { grantor: alice.app.agentId, claim: result.Ok })
    t.deepEqual(claim, { Ok: 'QmYsFu7QGaVeUUac1E4BWST7BR38cYvzRaaTc3YS9WqsTu' });

    // Bob can now create a post on alice's chain via a node-to-node message with the claim
    const post_content = "Holo world"
    const params = { grantor: alice.app.agentId, content: post_content, in_reply_to: null }
    const create_result = await bob.app.call("blog", "create_post_with_claim", params)
    t.deepEqual(create_result, {Ok: "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk"})

    // Confirm that the post was actually added to alice's chain
    const get_post_result = await alice.app.call("blog", "get_post", { post_address: create_result.Ok })
    const value = JSON.parse(get_post_result.Ok.App[1])
    t.equal(value.content, post_content)


    // Check that when bob tries to make this call it fails because there is no grant stored
    const params2 = { grantor: bob.app.agentId, content: post_content, in_reply_to: null }
    const create2_result = await bob.app.call("blog", "create_post_with_claim", params2)
    t.deepEqual(create2_result, {Ok: "error: no matching grant for claim"})

})

scenario('sign_and_verify_message', async (s, t, { alice, bob }) => {
    const message = "Hello everyone! Time to start the secret meeting";

    const SignResult = await bob.app.call("converse", "sign_message", { key_id:"", message: message });
    t.deepEqual(SignResult, { Ok: 'N4wF/U1By7TA4H3k9i+5r8IGYzRMTi59mr/XZOpz8Ydj85mbupUk6tHMf/owgjJo43Vu1lja7ZFKGOFQnpa0Bw==' });

    const provenance = [bob.app.agentId, SignResult.Ok];

    const VerificationResult = await alice.app.call("converse", "verify_message", { message, provenance });
    t.deepEqual(VerificationResult, { Ok: true });
})

scenario('encrypt_and_decrypt_message', async (s, t, { alice, bob }) => {
  const message = "Hello everyone! Time to start the secret meeting";

  const EncryptResult = await bob.app.call("simple", "encrypt", { payload:message });
  
  t.ok(EncryptResult);
  const DecryptResult = await alice.app.call("simple", "decrypt", { payload:EncryptResult.Ok });
  t.deepEqual(DecryptResult.Ok, message);
})

scenario('secrets', async (s, t, { alice }) => {
    const ListResult = await alice.app.call("converse", "list_secrets", { });
    // it should start out with the genesis made seed
    t.deepEqual(ListResult, { Ok: [ 'app_root_seed', 'primary_keybundle:enc_key', 'primary_keybundle:sign_key', 'root_seed' ]  });

    const AddSeedResult = await alice.app.call("converse", "add_seed", {src_id: "app_root_seed", dst_id: "app_seed:1", index: 1 });
    t.ok(AddSeedResult)

    const AddKeyResult = await alice.app.call("converse", "add_key", {src_id: "app_seed:1", dst_id: "app_key:1" });
    t.ok(AddKeyResult)

    const ListResult1 = await alice.app.call("converse", "list_secrets", { });
    // it should start out with the genesis made seed
    t.deepEqual(ListResult1, { Ok: [ 'app_key:1', 'app_root_seed', 'app_seed:1', 'primary_keybundle:enc_key', 'primary_keybundle:sign_key', 'root_seed' ]  });

    const message = "Hello everyone! Time to start the secret meeting";

    const SignResult = await alice.app.call("converse", "sign_message", { key_id:"app_key:1", message: message });
    t.ok(SignResult)

    // use the public key returned by add key as the provenance source
    const provenance = [AddKeyResult.Ok, SignResult.Ok];
    const VerificationResult = await alice.app.call("converse", "verify_message", { message, provenance });
    t.deepEqual(VerificationResult, { Ok: true });

    // use the agent key as the provenance source (which should fail)
    const provenance1 = [alice.app.agentId, SignResult.Ok];
    const VerificationResult1 = await alice.app.call("converse", "verify_message", { message, provenance: provenance1 });
    t.deepEqual(VerificationResult1, { Ok: false });

    const GetKeyResult = await alice.app.call("converse", "get_pubkey", {src_id: "app_key:1" });
    t.ok(GetKeyResult)
    t.deepEqual(GetKeyResult,AddKeyResult)

})

scenario('agentId', async (s, t, { alice, bob }) => {
  t.ok(alice.app.agentId)
  t.notEqual(alice.app.agentId, bob.app.agentId)
})

scenario('show_env', async (s, t, { alice }) => {
  const result = await alice.app.call("blog", "show_env", {})

  t.equal(result.Ok.dna_address, alice.app.dnaAddress)
  t.equal(result.Ok.dna_name, "HDK-spec-rust")
  t.equal(result.Ok.agent_address, alice.app.agentId)
  t.equal(result.Ok.agent_id, '{"nick":"alice::app","pub_sign_key":"' + alice.app.agentId + '"}')
  t.equal(result.Ok.properties, '{"test_property":"test-property-value"}')

  // don't compare the public token because it changes every time we change the dna.
  t.deepEqual(result.Ok.cap_request.provenance, [ alice.app.agentId, 'HFQkrDmnSOcmGQnYNtaYZWj89rlIQVFg0PpEoeFyx/Qw6Oizy5PI+tcsO8wYrllkuVPPzF5P3pvbCctKkfyGBg==' ]
);

})

scenario('get sources', async (s, t, { alice, bob, carol }) => {
  const params = { content: 'whatever', in_reply_to: null }
  const address = await alice.app.callSync('blog', 'create_post', params).then(x => x.Ok)
  const address1 = await alice.app.callSync('blog', 'create_post', params).then(x => x.Ok)
  const address2 = await bob.app.callSync('blog', 'create_post', params).then(x => x.Ok)
  const address3 = await carol.app.callSync('blog', 'create_post', params).then(x => x.Ok)
  t.equal(address, address1)
  t.equal(address, address2)
  t.equal(address, address3)
  const sources1 = (await alice.app.call('blog', 'get_sources', { address })).Ok.sort()
  const sources2 = (await bob.app.call('blog', 'get_sources', { address })).Ok.sort()
  const sources3 = (await carol.app.call('blog', 'get_sources', { address })).Ok.sort()
  // NB: alice shows up twice because she published the same entry twice
  const expected = [alice.app.agentId, alice.app.agentId, bob.app.agentId, carol.app.agentId].sort()
  t.deepEqual(sources1, expected)
  t.deepEqual(sources2, expected)
  t.deepEqual(sources3, expected)
})

scenario('cross zome call', async (s, t, { alice }) => {

  const num1 = 2
  const num2 = 2
  const params = { num1, num2 }
  const result = await alice.app.call("blog", "check_sum", params)
  t.notOk(result.Err)
  t.equal(result.Ok, 4)
})

scenario('send ping', async (s, t, { alice, bob }) => {
  const params = { to_agent: bob.app.agentId, message: "hello" }
  const result = await alice.app.call("blog", "ping", params)
    t.deepEqual(result, { Ok: { msg_type:"response", body: "got hello from HcSCIv3cPT5kegjoqgXM7nVU8rFbd9pyg5oOYUz9PSryp5mb7DKhCsXCS768pua" } })
})

scenario('hash_post', async (s, t, { alice }) => {

  const params = { content: "Holo world" }
  const result = await alice.app.call("blog", "post_address", params)

  t.equal(result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
})

scenario('hash_memo', async (s, t, { alice }) => {

  const params = { content: "Reminder: Buy some HOT." }
  const result = await alice.app.call("blog", "memo_address", params)

  t.equal(result.Ok, "QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i")
})

scenario('create_post', async (s, t, { alice }) => {

  const content = "Holo world"
  const in_reply_to = null
  const params = { content, in_reply_to }
  const result = await alice.app.call("blog", "create_post", params)

  t.ok(result.Ok)
  t.notOk(result.Err)
  t.equal(result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
})

scenario('create_post_malformed_args', async (s, t, { alice }) => {

  const content = "Holo world"
  const malformed_arg = content
  const params = { malformed_arg }
  const result = await alice.callSync("blog", "create_post", params)

  t.notOk(result.Ok)
  // TODO The result should have a deserialization error but this assertion instead fails!
  t.ok(result.Err)
})

scenario('create_tagged_post and retrieve all tags', async (s, t, { alice }) => {
  const result1 = await alice.app.callSync("blog", "create_tagged_post", {
    content: "Tutorial on amazing Holochain design patterns",
    tag: "work"
  })
  t.ok(result1.Ok)

  const result2 = await alice.app.callSync("blog", "create_tagged_post", {
    content: "Fly tying, is it for you?",
    tag: "fishing"
  })
  t.ok(result2.Ok)

  const getResult = await alice.app.callSync("blog", "my_posts", {})
  t.equal(getResult.Ok.links.length, 2)
  let tags = getResult.Ok.links.map(l => l.tag)
  t.ok(tags.includes("work"))
  t.ok(tags.includes("fishing"))
})

scenario('create_tagged_post and retrieve exact tag match', async (s, t, { alice }) => {
  const result1 = await alice.app.callSync("blog", "create_tagged_post", {
    content: "Tutorial on amazing Holochain design patterns",
    tag: "work"
  })
  t.ok(result1.Ok)

  const result2 = await alice.app.callSync("blog", "create_tagged_post", {
    content: "Fly tying, is it for you?",
    tag: "fishing"
  })
  t.ok(result2.Ok)

  const getResult = await alice.app.callSync("blog", "my_posts", {tag: "fishing"})
  t.equal(getResult.Ok.links.length, 1)
  let tags = getResult.Ok.links.map(l => l.tag)
  t.notOk(tags.includes("work"))
  t.ok(tags.includes("fishing"))
})

scenario('create_tagged_post and retrieve regex tag match', async (s, t, { alice }) => {
  const result1 = await alice.app.callSync("blog", "create_tagged_post", {
    content: "A post made on the 10th of October",
    tag: "10/10/2019"
  })
  t.ok(result1.Ok)

  const result2 = await alice.app.callSync("blog", "create_tagged_post", {
    content: "A post made on the 10th of September",
    tag: "10/9/2019"
  })
  t.ok(result2.Ok)

  const getResult = await alice.app.callSync("blog", "my_posts", {tag: "^10\/[0-9]+\/2019$"})
  t.equal(getResult.Ok.links.length, 2)
  let tags = getResult.Ok.links.map(l => l.tag)
  t.ok(tags.includes("10/10/2019"))
  t.ok(tags.includes("10/9/2019"))
})

scenario('tagged link validation', async (s, t, { alice }) => {
  const result1 = await alice.app.callSync("blog", "create_tagged_post", {
    content: "Achieving a light and fluffy texture",
    tag: "muffins"
  })
  t.ok(result1.Err)  // the linking of the entry should fail because `muffins` is the banned tag

  const getResult = await alice.app.callSync("blog", "my_posts", {})
  t.equal(getResult.Ok.links.length, 0)
})

scenario('create_post_countersigned', async (s, t, { alice, bob }) => {

  const content = "Holo world"
  const in_reply_to = null

  const address_params = { content }
  const address_result = await bob.app.call("blog", "post_address", address_params)

  t.ok(address_result.Ok)
  const SignResult = await bob.app.call("converse", "sign_message", { key_id:"", message: address_result.Ok });
  t.ok(SignResult.Ok)

  const counter_signature = [bob.app.agentId, SignResult.Ok];

  const params = { content, in_reply_to, counter_signature }
  const result = await alice.app.call("blog", "create_post_countersigned", params)

  t.ok(result.Ok)
  t.notOk(result.Err)
  t.equal(result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
})


scenario('create_memo', async (s, t, { alice }) => {

  const content = "Reminder: Buy some HOT."
  const params = { content }
  const result = await alice.app.call("blog", "create_memo", params)

  t.ok(result.Ok)
  t.notOk(result.Err)
  t.equal(result.Ok, "QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i")
})

scenario('my_memos', async (s, t, { alice }) => {

  const content = "Reminder: Buy some HOT."
  const params = { content }
  const create_memo_result = await alice.app.call("blog", "create_memo", params)

  t.ok(create_memo_result.Ok)
  t.notOk(create_memo_result.Err)
  t.equal(create_memo_result.Ok, "QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i")

  const my_memos_result = await alice.app.call("blog", "my_memos", {})

  t.ok(my_memos_result.Ok)
  t.notOk(my_memos_result.Err)
  t.deepEqual(my_memos_result.Ok, ["QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i"])
})


scenario('get_memo_returns_none', async (s, t, { alice, bob}) => {

  const content = "Reminder: Buy some HOT."
  const params = { content }
  const create_memo_result = await alice.app.call("blog", "create_memo", params)

  t.ok(create_memo_result.Ok)
  t.notOk(create_memo_result.Err)
  t.equal(create_memo_result.Ok, "QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i")

  const alice_get_memo_result = await alice.app.call("blog", "get_memo",
    { memo_address:create_memo_result.Ok })

  t.ok(alice_get_memo_result.Ok)
  t.notOk(alice_get_memo_result.Err)
  t.deepEqual(alice_get_memo_result.Ok,
    { App: [ 'memo', '{"content":"Reminder: Buy some HOT.","date_created":"now"}' ] })

  const bob_get_memo_result = await bob.app.call("blog", "get_memo",
    { memo_address:create_memo_result.Ok })

  t.equal(bob_get_memo_result.Ok, null)
  t.notOk(bob_get_memo_result.Err)

})

scenario('my_memos_are_private', async (s, t, { alice, bob }) => {

  const content = "Reminder: Buy some HOT."
  const params = { content }
  const create_memo_result = await alice.app.call("blog", "create_memo", params)

  t.ok(create_memo_result.Ok)
  t.notOk(create_memo_result.Err)
  t.equal(create_memo_result.Ok, "QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i")

  const alice_memos_result = await alice.app.call("blog", "my_memos", {})

  t.ok(alice_memos_result.Ok)
  t.notOk(alice_memos_result.Err)
  t.deepEqual(alice_memos_result.Ok,
    ["QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i"])

  const bob_memos_result = await bob.app.call("blog", "my_memos", {})

  t.ok(bob_memos_result.Ok)
  t.notOk(bob_memos_result.Err)
  t.deepEqual(bob_memos_result.Ok, [])

})


scenario('delete_post', async (s, t, { alice, bob }) => {

  //creates a simple link with alice as author with initial chain header
  await alice.app.callSync("simple", "create_link",
    { "base":alice.app.agentId, "target": "Posty" }
  )


  //creates a simple link with bob as author with different chain header
  await bob.app.callSync("simple", "create_link",
    { "base":alice.app.agentId, "target": "Posty" }
  )

  //get all created links so far alice
  const alice_posts = await bob.app.call("simple", "get_my_links",
    { "base": alice.app.agentId,"status_request" : "Live" }
  )


  //expect two links from alice
  t.ok(alice_posts.Ok)
  t.equal(alice_posts.Ok.links.length,2 );

  //get all created links so far for bob
  const bob_posts = await bob.app.call("simple", "get_my_links",
    { "base": alice.app.agentId,"status_request" : "Live" }
  )


  //expected two links from bob
  t.ok(bob_posts.Ok)
  t.equal(bob_posts.Ok.links.length,2 );

  //alice removes both links
  await alice.app.callSync("simple", "delete_link", { "base":alice.app.agentId, "target": "Posty" })

  // get links from bob
  const bob_agent_posts_expect_empty = await bob.app.call("simple", "get_my_links",{ "base": alice.app.agentId,"status_request" : "Live" })
  //get links from alice
  const alice_agent_posts_expect_empty = await alice.app.call("simple", "get_my_links",{ "base": alice.app.agentId,"status_request" : "Live" })

  //bob expects zero links
  t.ok(bob_agent_posts_expect_empty.Ok)
  t.equal(bob_agent_posts_expect_empty.Ok.links.length, 0);
  //alice expects zero alice
  t.ok(alice_agent_posts_expect_empty.Ok)
  t.equal(alice_agent_posts_expect_empty.Ok.links.length, 0);


  //different chain hash up to this point so we should be able to create a link with the same data
  await alice.app.callSync("simple", "create_link",{ "base":alice.app.agentId, "target": "Posty" })

  //get alice posts
  const alice_posts_not_empty = await bob.app.call("simple", "get_my_links",{ "base": alice.app.agentId,"status_request" : "Live" })

   //expect 1 post
  t.ok(alice_posts_not_empty.Ok)
  t.equal(alice_posts_not_empty.Ok.links.length, 1);


})

scenario('get_links_and_load with a delete_post', async (s, t, { alice }) => {

  //create post
  const alice_create_post_result = await alice.app.callSync("blog", "create_post",
    { "content": "Posty", "in_reply_to": "" }
  )

  const alice_get_post_result1 = await alice.app.callSync("blog", "my_posts_with_load",
    { "tag": null }
  )

  t.ok(alice_get_post_result1.Ok)
  t.equal(alice_get_post_result1.Ok.length, 1);

  //remove link by alicce
  await alice.app.callSync("blog", "delete_post", { "content": "Posty", "in_reply_to": "" })

  const alice_get_post_result2 = await alice.app.callSync("blog", "my_posts_with_load",
    { "tag": null }
  )
  t.ok(alice_get_post_result2.Ok)
  t.equal(alice_get_post_result2.Ok.length, 0);
})

scenario('delete_entry_post', async (s, t, { alice, bob }) => {
  const content = "Hello Holo world 321"
  const in_reply_to = null
  const params = { content, in_reply_to }

  //commit create_post
  const createResult = await alice.app.callSync("blog", "create_post", params)

  t.ok(createResult.Ok)


  //delete entry post
  const deletionParams = { post_address: createResult.Ok }
  const deletionResult = await alice.app.callSync("blog", "delete_entry_post", deletionParams)

  t.ok(deletionResult.Ok)


  //delete should fail
  const failedDelete = await bob.app.callSync("blog", "delete_entry_post", { post_address: createResult.Ok })
  t.deepEqual(failedDelete.Err, { Internal: 'Entry Could Not Be Found' });

  //get initial entry
  const GetInitialParamsResult = await bob.app.call("blog", "get_initial_post", { post_address: createResult.Ok })
  t.deepEqual(JSON.parse(GetInitialParamsResult.Ok.App[1]), { content: "Hello Holo world 321", date_created: "now" });

  const entryWithOptionsGet = { post_address: createResult.Ok }
  const entryWithOptionsGetResult = await bob.app.call("blog", "get_post_with_options", entryWithOptionsGet);
  t.deepEqual(JSON.parse(entryWithOptionsGetResult.Ok.result.All.items[0].entry.App[1]), { content: "Hello Holo world 321", date_created: "now" })
})

scenario('update_entry_validation', async (s, t, { alice, bob }) => {
  //update entry does not exist
  const updateParams = { post_address: "1234", new_content: "Hello Holo V2" }
  const UpdateResult = await bob.app.callSync("blog", "update_post", updateParams)

  t.deepEqual(UpdateResult.Err, { Internal: 'failed to update post' });

  const content = "Hello Holo world 321"
  const in_reply_to = null
  const params = { content, in_reply_to }

  //commit create_post
  const createResult = await alice.app.callSync("blog", "create_post", params)

  t.ok(createResult.Ok)

  const updateParamsV2 = { post_address: createResult.Ok, new_content: "Hello Holo world 321" }
  const UpdateResultV2 = await bob.app.callSync("blog", "update_post", updateParamsV2)
  t.deepEqual(JSON.parse(UpdateResultV2.Err.Internal).kind.ValidationFailed, "Trying to modify with same data");


})

scenario('update_post', async (s, t, { alice, bob }) => {
  const content = "Hello Holo world 123"
  const in_reply_to = null
  const params = { content, in_reply_to }

  //commit version 1
  const createResult = await alice.app.callSync("blog", "create_post", params)
  t.ok(createResult.Ok)

  //get v1
  const updatedPostV1 = await alice.app.call("blog", "get_post", { post_address: createResult.Ok })
  const UpdatePostV1Content = { content: "Hello Holo world 123", date_created: "now" };
  t.ok(updatedPostV1.Ok)
  t.deepEqual(JSON.parse(updatedPostV1.Ok.App[1]), UpdatePostV1Content)

  //update to version 2
  const updatePostContentV2 = { content: "Hello Holo V2", date_created: "now" };
  const updateParamsV2 = { post_address: createResult.Ok, new_content: "Hello Holo V2" }
  const UpdateResultV2 = await bob.app.callSync("blog", "update_post", updateParamsV2)
  t.ok(UpdateResultV2.Ok)
  t.notOk(UpdateResultV2.Err)

  //get v2 using initial adderss
  const updatedPostv2Initial = await alice.app.call("blog", "get_post", { post_address: createResult.Ok })
  t.ok(updatedPostv2Initial.Ok)
  t.notOk(updatedPostv2Initial.Err)
  t.deepEqual(JSON.parse(updatedPostv2Initial.Ok.App[1]), updatePostContentV2)

  //get v2 latest address
  const updatedPostv2Latest = await alice.app.call("blog", "get_post", { post_address: UpdateResultV2.Ok })
  t.ok(updatedPostv2Latest.Ok)
  t.notOk(updatedPostv2Latest.Err)
  t.deepEqual(JSON.parse(updatedPostv2Latest.Ok.App[1]), updatePostContentV2)


  //get v1 using initial address
  const GetInitialPostV1Initial = await alice.app.call("blog", "get_initial_post", { post_address: createResult.Ok })
  t.ok(GetInitialPostV1Initial.Ok)
  t.notOk(GetInitialPostV1Initial.Err)
  t.deepEqual(JSON.parse(GetInitialPostV1Initial.Ok.App[1]), { content: "Hello Holo world 123", date_created: "now" })

  //get v2 latest address
  const GetInitialPostV2Latest = await alice.app.call("blog", "get_initial_post", { post_address: UpdateResultV2.Ok })
  t.ok(GetInitialPostV2Latest.Ok)
  t.notOk(GetInitialPostV2Latest.Err)
  t.deepEqual(JSON.parse(GetInitialPostV2Latest.Ok.App[1]), updatePostContentV2)

  //update to version 3
  const UpdatePostV3Content = { content: "Hello Holo V3", date_created: "now" };
  const updateParamsV3 = { post_address: createResult.Ok, new_content: "Hello Holo V3" }
  const UpdateResultV3 = await alice.app.callSync("blog", "update_post", updateParamsV3)
  t.ok(UpdateResultV3.Ok)
  t.notOk(UpdateResultV3.Err)

  //get v3 using initial adderss
  const updatedPostV3Initial = await alice.app.call("blog", "get_post", { post_address: createResult.Ok })
  t.ok(updatedPostV3Initial.Ok)
  t.notOk(updatedPostV3Initial.Err)
  t.deepEqual(JSON.parse(updatedPostV3Initial.Ok.App[1]), UpdatePostV3Content)

  //get v3 using address of v2
  const updatedPostV3Latest = await alice.app.call("blog", "get_post", { post_address: UpdateResultV2.Ok })
  t.ok(updatedPostV3Latest.Ok)
  t.notOk(updatedPostV3Latest.Err)
  t.deepEqual(JSON.parse(updatedPostV3Latest.Ok.App[1]), UpdatePostV3Content)

  //update to version 4
  const updatePostV4Content = { content: "Hello Holo V4", date_created: "now" };
  const updateParamsV4 = { post_address: createResult.Ok, new_content: "Hello Holo V4" }
  const UpdateResultV4 = await alice.app.callSync("blog", "update_post", updateParamsV4)
  t.notOk(UpdateResultV4.Err)
  t.ok(UpdateResultV4.Ok)

  //get history entry v4
  const entryHistoryV4Params = { post_address: UpdateResultV4.Ok }
  const entryHistoryV4 = await alice.app.call("blog", "get_history_post", entryHistoryV4Params)
  t.ok(UpdateResultV4.Ok)
  t.notOk(UpdateResultV4.Err)
  t.deepEqual(entryHistoryV4.Ok.items.length, 1);
  t.deepEqual(JSON.parse(entryHistoryV4.Ok.items[0].entry.App[1]), updatePostV4Content);
  t.deepEqual(entryHistoryV4.Ok.items[0].meta.address, UpdateResultV4.Ok);
  t.deepEqual(entryHistoryV4.Ok.items[0].meta.crud_status, "live");

  //get history entry all
  const entryHistoryAllParams = { post_address: createResult.Ok }
  const entryHistoryAll = await alice.app.call("blog", "get_history_post", entryHistoryAllParams)

  t.deepEqual(entryHistoryAll.Ok.items.length, 4);
  t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[0].entry.App[1]), { content: "Hello Holo world 123", date_created: "now" });
  t.deepEqual(entryHistoryAll.Ok.items[0].meta.address, createResult.Ok);
  t.deepEqual(entryHistoryAll.Ok.items[0].meta.crud_status, "modified");
  t.deepEqual(entryHistoryAll.Ok.crud_links[createResult.Ok], UpdateResultV2.Ok)

  t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[1].entry.App[1]), updatePostContentV2);
  t.deepEqual(entryHistoryAll.Ok.items[1].meta.address, UpdateResultV2.Ok);
  t.deepEqual(entryHistoryAll.Ok.items[1].meta.crud_status, "modified");
  t.deepEqual(entryHistoryAll.Ok.crud_links[UpdateResultV2.Ok], UpdateResultV3.Ok)

  t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[2].entry.App[1]), UpdatePostV3Content);
  t.deepEqual(entryHistoryAll.Ok.items[2].meta.address, UpdateResultV3.Ok);
  t.deepEqual(entryHistoryAll.Ok.items[2].meta.crud_status, "modified");
  t.deepEqual(entryHistoryAll.Ok.crud_links[UpdateResultV3.Ok], UpdateResultV4.Ok)

  t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[3].entry.App[1]), updatePostV4Content);
  t.deepEqual(entryHistoryAll.Ok.items[3].meta.address, UpdateResultV4.Ok);
  t.deepEqual(entryHistoryAll.Ok.items[3].meta.crud_status, "live");
  t.notOk(entryHistoryAll.Ok.crud_links[UpdateResultV4.Ok])

  const entryWithOptionsGet = { post_address: createResult.Ok }
  const entryWithOptionsGetResult = await alice.app.call("blog", "get_post_with_options_latest", entryWithOptionsGet);

  t.deepEqual(JSON.parse(entryWithOptionsGetResult.Ok.App[1]), updatePostV4Content);
})


scenario('remove_update_modifed_entry', async (s, t, { alice, bob }) => {
  const content = "Hello Holo world 123"
  const in_reply_to = null
  const params = { content, in_reply_to }

  //commit version 1
  const createResult = await alice.app.callSync("blog", "create_post", params)
  t.ok(createResult.Ok)
  //get entry
  const updatedPostV1 = await alice.app.call("blog", "get_post", { post_address: createResult.Ok })
  t.ok(updatedPostV1.Ok)
  t.deepEqual(JSON.parse(updatedPostV1.Ok.App[1]), { content: "Hello Holo world 123", date_created: "now" })

  //delete
  const removeParamsV2 = { post_address: createResult.Ok }
  const removeResultV2 = await bob.app.callSync("blog", "delete_entry_post", removeParamsV2)
  t.ok(removeResultV2.Ok)

  //get v2 using initial adders
  const Postv2Initial = await alice.app.call("blog", "get_initial_post", { post_address: createResult.Ok })
  t.ok(Postv2Initial.Ok)
  t.deepEqual(JSON.parse(Postv2Initial.Ok.App[1]), { content: "Hello Holo world 123", date_created: "now" })

  //failed delete
  const failedDelete = await alice.app.callSync("blog", "delete_entry_post", { post_address: createResult.Ok })
  t.deepEqual(failedDelete.Err, { Internal: 'Entry Could Not Be Found' });
})

scenario('create_post with bad reply to', async (s, t, { alice }) => {
  const content = "Holo world"
  const in_reply_to = "bad"
  const params = { content, in_reply_to }
  const result = await alice.app.call("blog", "create_post", params)

  // bad in_reply_to is an error condition
  t.ok(result.Err)
  t.notOk(result.Ok)
  const error = JSON.parse(result.Err.Internal)
  t.deepEqual(error.kind, { ErrorGeneric: "Base for link not found" })
  t.ok(error.file)
  t.ok(error.line)
})

scenario('delete_post_with_bad_link', async (s, t, { alice, bob }) => {

  const result_bob_delete = await bob.app.callSync("blog", "delete_post", {
    "content": "Bad"
  })

  // bad in_reply_to is an error condition
  t.ok(result_bob_delete.Err)
  t.notOk(result_bob_delete.Ok)
  const error = JSON.parse(result_bob_delete.Err.Internal)
  t.deepEqual(error.kind, { ErrorGeneric: "Target for link not found" })
  t.ok(error.file)
  t.ok(error.line)
})

scenario('post max content size 280 characters', async (s, t, { alice }) => {

  const content = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum."
  const in_reply_to = null
  const params = { content, in_reply_to }
  const result = await alice.app.call("blog", "create_post", params)

  // result should be an error
  t.ok(result.Err);
  t.notOk(result.Ok)

  const inner = JSON.parse(result.Err.Internal)

  t.ok(inner.file)
  t.deepEqual(inner.kind, { "ValidationFailed": "Content too long" })
  t.ok(inner.line)
})

scenario('posts_by_agent', async (s, t, { alice }) => {

  const agent = "Bob"
  const params = { agent }

  const result = await alice.app.call("blog", "posts_by_agent", params)

  t.deepEqual(result.Ok, { links: [] })
})

scenario('my_posts', async (s, t, { alice }) => {

  await alice.app.callSync("blog", "create_post",
    { "content": "Holo world", "in_reply_to": "" }
  )

  await alice.app.callSync("blog", "create_post",
    { "content": "Another post", "in_reply_to": "" }
  )

  const result = await alice.app.call("blog", "my_posts", {})

  t.equal(result.Ok.links.length, 2)
})


scenario('my_posts_immediate_timeout', async (s, t, { alice }) => {

  await alice.app.call("blog", "create_post",
    { "content": "Holo world", "in_reply_to": "" }
  )

  const result = await alice.app.call("blog", "my_posts_immediate_timeout", {})

  t.ok(result.Err)
  console.log(result)
  t.equal(JSON.parse(result.Err.Internal).kind, "Timeout")
})

scenario('get_sources_from_link', async (s, t, { alice, bob }) => {

  await alice.app.callSync("blog", "create_post", {
    "content": "Holo world", "in_reply_to": null
  });

  await bob.app.callSync("blog", "create_post", {
    "content": "Another one", "in_reply_to": null
  });
  const alice_posts = await bob.app.call("blog","authored_posts_with_sources", {
    "agent" : alice.app.agentId
  });

  const bob_posts = await alice.app.call("blog","authored_posts_with_sources", {
    "agent" : bob.app.agentId
  });

  t.equal(bob_posts.Ok.links.length, 1)
  t.equal(bob.app.agentId,bob_posts.Ok.links[0].headers[0].provenances[0][0]);
  t.equal(alice_posts.Ok.links.length, 1)
  t.equal(alice.app.agentId,alice_posts.Ok.links[0].headers[0].provenances[0][0]);

})

scenario('get_sources_after_same_link', async (s, t, { alice, bob }) => {

  await bob.app.callSync("blog", "create_post_with_agent",
    { "agent_id": alice.app.agentId ,"content": "Holo world", "in_reply_to": null }
  );
  await bob.app.callSync("blog", "create_post_with_agent",
  { "agent_id": alice.app.agentId ,"content": "Holo world", "in_reply_to": null }
  );

  const alice_posts = await bob.app.call("blog","authored_posts_with_sources",
  {
    "agent" : alice.app.agentId
  });
  const bob_posts = await alice.app.call("blog","authored_posts_with_sources",
  {
    "agent" : alice.app.agentId
  });

  t.equal(bob.app.agentId,alice_posts.Ok.links[0].headers[0].provenances[0][0]);
  t.equal(bob.app.agentId,bob_posts.Ok.links[0].headers[0].provenances[0][0]);

})


scenario('get_links_crud', async (s, t, { alice, bob }) => {

  //commits an entry and creates two links for alice
  await alice.app.callSync("simple", "create_link",
    { "base": alice.app.agentId ,"target": "Holo world" }
  );
  const alice_result = await alice.app.callSync("simple", "create_link",
  { "base": alice.app.agentId ,"target": "Holo world 2" }
  );

  //get posts for alice from alice
  const alice_posts_live= await alice.app.call("simple","get_my_links",
  {
    "base" : alice.app.agentId,"status_request":"Live"
  })
  console.log("alice posts" + JSON.stringify(alice_posts_live));

  //get posts for alice from bob
  const bob_posts_live= await bob.app.call("simple","get_my_links",
  {
    "base" : alice.app.agentId,
    "status_request":"Live"
  })

  //make sure all our links are live and they are two of them
  t.equal(2,alice_posts_live.Ok.links.length);
  t.equal("live",alice_posts_live.Ok.links[0].status);
  t.equal("live",alice_posts_live.Ok.links[1].status);
  t.equal(2,bob_posts_live.Ok.links.length);
  t.equal("live",bob_posts_live.Ok.links[0].status);
  t.equal("live",bob_posts_live.Ok.links[1].status);

  ////delete the holo world post from the links alice created
  await alice.app.callSync("simple","delete_link",
  {
    "base" : alice.app.agentId,
    "target" : "Holo world"
  });

  //get all posts with a deleted status from bob
  const bob_posts_deleted = await bob.app.call("simple","get_my_links",
  {
    "base" : alice.app.agentId,
    "status_request" : "Deleted"
  });

  // get all posts with a deleted status from alice
  const alice_posts_deleted = await alice.app.call("simple","get_my_links",
  {
    "base" : alice.app.agentId,
    "status_request" : "Deleted"
  });

  //make sure only 1 is returned and it has a status of deleted
  t.equal(1,alice_posts_deleted.Ok.links.length);
  t.equal(1,bob_posts_deleted.Ok.links.length);
  t.equal("deleted",alice_posts_deleted.Ok.links[0].status);
  t.equal("deleted",bob_posts_deleted.Ok.links[0].status);

  //get all posts from the agent
  const bob_posts_all = await bob.app.call("simple","get_my_links",
  {
    "base" : alice.app.agentId,
    "status_request" : "All"

  });
  const alice_posts_all = await alice.app.call("simple","get_my_links",
  {
    "base" : alice.app.agentId,
    "status_request" : "All"
  });

  //make sure we get two links with the first one being a live link and the second one being a deleted link
  t.equal(2,alice_posts_all.Ok.links.length);
  t.equal("live",alice_posts_all.Ok.links[0].status);
  t.equal("deleted",alice_posts_all.Ok.links[1].status);
  t.equal(2,bob_posts_all.Ok.links.length);
  t.equal("live",bob_posts_all.Ok.links[0].status);
  t.equal("deleted",bob_posts_all.Ok.links[1].status);


})

scenario('get_links_crud_count', async (s, t, { alice, bob }) => {

  //commits an entry and creates two links for alice
  await alice.app.callSync("simple", "create_link_with_tag",
    { "base": alice.app.agentId ,"target": "Holo world","tag":"tag" }
  );

  //commit an entry with other tag
  await alice.app.callSync("simple", "create_link_with_tag",
  { "base": alice.app.agentId ,"target": "Holo world", "tag":"differen" }
   );
  
  await alice.app.callSync("simple", "create_link_with_tag",
  { "base": alice.app.agentId ,"target": "Holo world 2","tag":"tag" });

  //get posts for alice from alice
  const alice_posts_live= await alice.app.call("simple","get_my_links_count",
  {
    "base" : alice.app.agentId,
    "status_request":"Live",
    "tag":"tag"
  })

  //get posts for alice from bob
  const bob_posts_live= await bob.app.call("simple","get_my_links_count",
  {
    "base" : alice.app.agentId,
    "status_request":"Live",
    "tag":"tag"
  })

 
  
  //make sure count equals to 2
  t.equal(2,alice_posts_live.Ok.count);
  t.equal(2,bob_posts_live.Ok.count);

  const bob_posts_live_diff_tag= await bob.app.call("simple","get_my_links_count",
  {
    "base" : alice.app.agentId,
    "status_request":"Live",
    "tag":"differen"
  })

  t.equal(1,bob_posts_live_diff_tag.Ok.count);


  ////delete the holo world post from the links alice created
  await alice.app.callSync("simple","delete_link_with_tag",
  {
    "base" : alice.app.agentId,
    "target" : "Holo world",
    "tag":"tag"
  });

  //get all bob posts
  const bob_posts_deleted = await bob.app.call("simple","get_my_links_count",
  {
    "base" : alice.app.agentId,
    "status_request" : "Deleted",
    "tag":"tag"
  });

  // get all posts with a deleted status from alice
  const alice_posts_deleted = await alice.app.call("simple","get_my_links_count",
  {
    "base" : alice.app.agentId,
    "status_request" : "Deleted",
    "tag":"tag"
  });

  //make sure count is equal to 1
  t.equal(1,alice_posts_deleted.Ok.count);
  t.equal(1,bob_posts_deleted.Ok.count);

  const bob_posts_deleted_diff_tag= await bob.app.call("simple","get_my_links_count",
  {
    "base" : alice.app.agentId,
    "status_request":"Live",
    "tag":"differen"
  })

  t.equal(1,bob_posts_deleted_diff_tag.Ok.count);

})



scenario('create/get_post roundtrip', async (s, t, { alice }) => {
  const content = "Holo world"
  const in_reply_to = null
  const params = { content, in_reply_to }
  const create_post_result = await alice.app.call("blog", "create_post", params)
  const post_address = create_post_result.Ok

  const params_get = { post_address }
  const result = await alice.app.call("blog", "get_post", params_get)

  const entry_value = JSON.parse(result.Ok.App[1])
  t.comment("get_post() entry_value = " + entry_value + "")
  t.equal(entry_value.content, content)
  t.equal(entry_value.date_created, "now")

})

scenario('get_post with non-existant address returns null', async (s, t, { alice }) => {

  const post_address = "RANDOM"
  const params_get = { post_address }
  const result = await alice.app.call("blog", "get_post", params_get)

  // should be Ok value but null
  // lookup did not error
  // successfully discovered the entry does not exity
  const entry = result.Ok
  t.same(entry, null)
})

scenario('scenario test create & publish post -> get from other instance', async (s, t, { alice, bob }) => {

  const initialContent = "Holo world"
  const params = { content: initialContent, in_reply_to: null }
  const create_result = await alice.app.callSync("blog", "create_post", params)

  const params2 = { content: "post 2", in_reply_to: null }
  const create_result2 = await bob.app.callSync("blog", "create_post", params2)

  t.equal(create_result.Ok.length, 46)
  t.equal(create_result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")

  const post_address = create_result.Ok
  const params_get = { post_address }

  const result = await bob.app.call("blog", "get_post", params_get)
  const value = JSON.parse(result.Ok.App[1])
  t.equal(value.content, initialContent)
})

scenario('request grant', async (s, t, { alice, bob }) => {

    /*
      This is not a complete test of requesting a grant because currently there
      is no way in the test conductor to actually pass in the provenance of the
      call.  That will be added when we convert the test framework to being built
      on top of the rust conductor.   For now this is more a placeholder test, but
      note that the value returned is actually the capbability token value.
    */
    const result = await alice.app.call("blog", "request_post_grant", {})
    t.ok(result.Ok)
    t.notOk(result.Err)

    const grants = await alice.app.call("blog", "get_grants", {})
    t.ok(grants.Ok)
    t.notOk(grants.Err)

    t.equal(result.Ok, grants.Ok[0])
})

  scenario('emit signal', async (s, t, { alice }) => {
    const result = await alice.app.callSync("simple", "test_emit_signal", {message: "test message"})
    t.equal(alice.app.signals.length, 1)
    t.deepEqual(alice.app.signals[0], { signal_type: 'User', name: 'test-signal', arguments: '{"message":"test message"}' })
    t.notOk(result.Err)
  })

}
