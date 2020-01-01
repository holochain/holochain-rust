# walkman

For recording and playing back sim2h sessions:

```bash
# 1. Run a sim2h server with walkman logging turned on
HOLOCHAIN_WALKMAN_SIM2H=1 sim2h_server -p 9000 |& tee sim2h.log

# 2. Run a test which hits the sim2h_server with a lot of traffic
./run-a-real-e2e-test.sh

# 3. You can kill the sim2h server when the stress test is over

# 4. Condense the potentially huge log down into a minimal set of messages to replay
walkman cassette compile --path sim2h.log > sim2h.cassette

# 5. You can view the cassette in a slightly more readable form like so:
walkman cassette show --path sim2h.cassette

# 6. Start up the sim2h server again (no need to use the env var)
sim2h_server -p 9000 |& tee sim2h-playback.log

# 7. Playback the cassette with the server you just started up as a target
walkman playback sim2h --path sim2h.cassette --url ws://localhost:9002 |& tee cassette-playback.log
```
