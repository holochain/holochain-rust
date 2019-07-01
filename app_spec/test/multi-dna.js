module.exports = scenario => {

scenario('scenario test create & publish -> getting post via bridge (multi dna)', async (s, t, {alice, bob}) => {

  const initialContent = "Holo world"
  const params = { content: initialContent, in_reply_to: null }
  const create_result = await bob.callSync("blog", "create_post", params)

  t.equal(create_result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
  t.notEqual(alice.dnaAddress, bob.dnaAddress)

  const post_address = create_result.Ok
  const params_get = { post_address }

  const result = await alice.call("blog", "get_post_bridged", params_get)
  console.log("BRIDGE CALL RESULT: " + JSON.stringify(result))
  const value = JSON.parse(result.Ok.App[1])
  t.equal(value.content, initialContent)
})

}
