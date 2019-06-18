module.exports = scenario => {


scenario('can run a test with many agents', async (s, t, instanceMap) => {
  const nodes = Object.values(instanceMap)
  const anchor = nodes[0]

  t.ok(anchor.agentAddress)

  for (const [name, node] of Object.entries(nodes)) {
    await node.call('simple', 'create_link', {
      base: anchor.agentAddress,
      target: node.agentAddress,
    })
  }
  await s.consistent()
  const links = await anchor.call('simple', 'get_my_links', {
    base: anchor.agentAddress,
  })

  console.log('linx', links.Ok.links)

  t.ok(links.Ok)
  t.equal(Object.values(links.Ok.links).length, nodes.length)
})


}