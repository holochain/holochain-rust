module.exports = scenario => {

const delay = ms => new Promise(resolve => setTimeout(resolve, ms))

scenario('3 nodes share links', async (s, t, {alice, bob, carol}) => {

  await s.spawn(alice)
  await s.spawn(bob)
  await s.spawn(carol)

  const aliceId = alice.conductor.instanceMap.app.agentAddress
  // console.log('aliceID', alice.conductor.instanceMap.app)

  const create1 = await alice.conductor.instanceMap.app.call("simple","create_link",
    { "base": aliceId, "target": "Posty" }
  )
  await s.consistent()
  t.ok(create1.Ok)

  const bobLinks = await bob.conductor.instanceMap.app.call("simple", "get_my_links",
    { "base": aliceId, "status_request" : "Live" }
  )
  console.log('=============')
  console.log('bobLinks', bobLinks)
  t.ok(bobLinks.Ok)
  t.equal(bobLinks.Ok.links.length, 1)

  const carolLinks = await carol.conductor.instanceMap.app.call("simple", "get_my_links",
    { "base": aliceId, "status_request" : "Live" }
  )
  console.log('=============')
  console.log('carolLinks', carolLinks)
  t.ok(carolLinks.Ok)
  t.equal(carolLinks.Ok.links.length, 1)

})

scenario('offline node gets links gossiped to them when coming online', async (s, t, {alice, bob, carol}) => {

  await s.spawn(alice, bob)

  const aliceId = alice.conductor.instanceMap.app.agentAddress
  // console.log('aliceID', alice.conductor.instanceMap.app)

  const create1 = await alice.conductor.instanceMap.app.call("simple","create_link",
    { "base": aliceId, "target": "Posty" }
  )
  await s.consistent()
  t.ok(create1.Ok)

  await s.kill(alice)
  await delay(5000)

  const bobLinks = await bob.conductor.instanceMap.app.call("simple", "get_my_links",
    { "base": aliceId, "status_request" : "Live" }
  )
  console.log('=============')
  console.log('bobLinks', bobLinks)
  t.ok(bobLinks.Ok)
  t.equal(bobLinks.Ok.links.length, 1)


  await s.spawn(carol)
  await delay(10000)

  const carolLinks = await carol.conductor.instanceMap.app.call("simple", "get_my_links",
    { "base": aliceId, "status_request" : "Live" }
  )
  console.log('=============')
  console.log('carolLinks', carolLinks)
  t.ok(carolLinks.Ok)
  t.equal(carolLinks.Ok.links.length, 1)

})

}