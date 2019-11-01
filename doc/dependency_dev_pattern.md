# Pattern for new dev on holochain-rust dependencies (such as lib3h)

Holochain depends on many crates that are external to this monorepo. Holochain developers contribute to a number of these external crates. Many are part of [lib3h](https://github.com/holochain/lib3h), and there are others such as [hcid](https://github.com/holochain/hcid). This document includes some hints and best-practices for developing and testing across multiple repositories.

## `.cargo/config` "paths" Array

Cargo provides a handy tool for causing a build to look at a local checkout of a dependency instead of pulling it from crates.io.

Inside your local `holochain-rust` checkout, create a directory (if it doesn't exist) called `.cargo`. Inside this `.cargo` directory, create a file named `config`. The contents of this file are expected by cargo to be `toml`.

If this config file doesn't already contain a `paths` attribute (array) at the top, go ahead and add it:

`holochain-rust/.cargo/config`
```toml
# beginning of file
paths = [
  "../lib3h/sodium",
]
# any additional cargo config attributes
```

Depending on how the cargo dependency graph gets modified by the substitution, you may get a cargo warning. Despite this, it is still the easiest way to develop on holochain-rust dependencies. If the cargo maintainers ever make this a hard error as they are threatening to, we'll have to fall back to putting `[patch.crates-io]` entries in our workspace `Cargo.toml` and make sure we don't accidently commit them to git.

## Repo Feature Branches and crates.io

For now this can be a manual process. We may wish to automate some or all of this in the future:

1. Create a feature branch in dependant repo.
2. Create a feature branch with the same name in `holochain-rust` repo.
3. Develop && WIP PR in both.
4. Dependant repo PR is approved / merged.
5. Bump version in dependant repo && publish to crates.io.
6. Update version in `holochain-rust` && test. !! Make sure to remove your `.cargo/config` `"paths"` override !!
7. `holochain-rust` repo PR is approved / merged.
