# Uninstalling holochain-proto
If you previously installed holochain_proto, you may want to uninstall it before you install holochain-rust. Here's how to do it.
This was compiled from the answers at https://stackoverflow.com/questions/13792254/removing-packages-installed-with-go-get

##Option 1
1. Find the source directory under `$GOPATH/src` (by default, $GOPATH is `$HOME/go` on Unix-like systems and `%USERPROFILE%\go` on Windows)
1. Delete `$GOPATH/src/github.com/holochain`
1. Delete `$GOPATH/pkg/<architecture>/github.com/holochain`
1. Delete the following file under `$GOPATH/bin/`

`hcadmin`
`hcd`
`hcdev`
`bs`

1. execute `go clean` with the -n flag to do a dry run and see what it will do:

`go clean -i -n github.com/holochain/holochain-proto...`

1. execute `go clean` for real

`go clean -i github.com/holochain/holochain-proto...`
