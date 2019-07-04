module.exports = scenario => {

scenario('scenario test create & publish -> getting post via bridge (multi dna)', async (s, t, {conductor}) => {

  const initialContent = "Holo world"
  const params = { content: initialContent, in_reply_to: null }
  const create_result = await conductor.app2.callSync("blog", "create_post", params)

  t.equal(create_result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
  t.notEqual(conductor.app1.dnaAddress, conductor.app2.dnaAddress)

  const post_address = create_result.Ok
  const params_get = { post_address }

  const result = await conductor.app1.call("blog", "get_post_bridged", params_get)
  console.log("BRIDGE CALL RESULT: " + JSON.stringify(result))
  const value = JSON.parse(result.Ok.App[1])
  t.equal(value.content, initialContent)
})

}
