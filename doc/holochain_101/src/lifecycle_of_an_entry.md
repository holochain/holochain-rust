# Lifecycle of an Entry

## Commit

```
New entry
Validate commit
Commit to source chain
```

Entries must be committed to the local source chain before they can be broadcast
to the DHT.

Every entry must pass a ValidateCommit lifecycle function check before it can be
committed.

If ValidateCommit is not implemented for the zome committing the entry then this
is treated as a pass and the entry will be committed.
