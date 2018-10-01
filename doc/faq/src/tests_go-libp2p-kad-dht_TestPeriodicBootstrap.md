```golang
func TestPeriodicBootstrap(t *testing.T) {
	// Create a DHT with 30 nodes and run bootstrap a few times to populate the routing tables
  //   
  // example output: https://visual.tools/static/pastebin/TestPeriodicBootstrap.html

  // t.Skip("skipping test to debug another")
	fmt.Println("dht_test.go: TestPeriodicBootstrap")


	if ci.IsRunning() {
		t.Skip("skipping on CI. highly timing dependent")
	}

	ctx := context.Background()

	nDHTs := 30
	_, _, dhts := setupDHTS(ctx, nDHTs, t)
	defer func() {
		for i := 0; i < nDHTs; i++ {
			dhts[i].Close()
			defer dhts[i].host.Close()
		}
	}()

	// signal amplifier
	amplify := func(signal chan time.Time, other []chan time.Time) {
		for t := range signal {
			for _, s := range other {
				s <- t
			}
		}
		for _, s := range other {
			close(s)
		}
	}

	signal := make(chan time.Time)
	allSignals := []chan time.Time{}

	var cfg BootstrapConfig
	cfg = DefaultBootstrapConfig
	cfg.Queries = 5

	// kick off periodic bootstrappers with instrumented signals.
	for _, dht := range dhts {
		s := make(chan time.Time)
		allSignals = append(allSignals, s)
		dht.BootstrapOnSignal(cfg, s)
	}
	go amplify(signal, allSignals)

	log.Debugf("dhts are not connected. %d", nDHTs)
	for _, dht := range dhts {
		rtlen := dht.routingTable.Size()
		if rtlen > 0 {
			log.Errorf("routing table for %s should have 0 peers. has %d", dht.self, rtlen)
		}
	}

	for i := 0; i < nDHTs; i++ {
		connect(t, ctx, dhts[i], dhts[(i+1)%len(dhts)])
	}

	log.Debugf("DHTs are now connected to 1-2 others. %d", nDHTs)
	for _, dht := range dhts {
		rtlen := dht.routingTable.Size()
		if rtlen > 2 {
			log.Errorf("routing table for %s should have at most 2 peers. has %d", dht.self, rtlen)
		}
	}

  fmt.Printf("\n\ndht_test.go: TestPeriodicBootstrap: Outputting routing tables BEFORE bootstrap\n")
	printRoutingTables(dhts)

	fmt.Printf("\n\ndht_test.go: TestPeriodicBootstrap: Firing bootstrap starting signal\n")
  log.Debugf("bootstrapping them so they find each other. %d", nDHTs)
	signal <- time.Now()

	// this is async, and we dont know when it's finished with one cycle, so keep checking
	// until the routing tables look better, or some long timeout for the failure case.
	waitForWellFormedTables(t, dhts, 7, 10, 20*time.Second)

	fmt.Printf("\n\ndht_test.go: TestPeriodicBootstrap: Outputting routing tables AFTER bootstrap\n")
  printRoutingTables(dhts)
}
```

test output:
https://visual.tools/static/pastebin/TestPeriodicBootstrap.html