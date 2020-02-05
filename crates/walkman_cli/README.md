# walkman

For recording and playing back sim2h sessions:

```bash
# 0. Build walkman CLI (no nix command for this yet)
cargo install -f --path crates/walkman_cli

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
walkman playback sim2h --path sim2h.cassette --url ws://localhost:9000 |& tee cassette-playback.log
```

Steps 1-4 are the recording process, which only has to happen once. When iteratively testing, you can repeat the playback by running steps 6 and 7 (the last two steps) with the same cassette over and over.

## Installation

```
cargo install --force --path crates/walkman_cli
```
## Editing
If you would like to edit the cassette manually (e.g. change a timestamp) you will need to convert it to and from raw format.
```bash
# 1. Convert to raw
walkman cassette raw --path sim2h.cassette > sim2h.txt

# 2. Edit the text file.

# 3. Compile the raw text file.
walkman cassette compile-raw --path sim2h.txt > sim2h.cassette
```
