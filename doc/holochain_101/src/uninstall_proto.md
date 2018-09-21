# Uninstalling holochain-proto
If you previously installed holochain_proto, you may want to uninstall it before you install holochain-rust. Here's how to do it.
This was compiled from the answers at https://stackoverflow.com/questions/13792254/removing-packages-installed-with-go-get

1. Find the source directory under $GOPATH/src (by default, $GOPATH is $HOME/go on Unix-like systems and %USERPROFILE%\go on Windows)
2. Delete $GOPATH/src/github.com/holochain
3. Find the compiled package file under $GOPATH/pkg/<archictecture> (for example darwin_amd64).
4. Delete $GOPATH/pkg/<architecture>/github.com/holochain
5. execute this command:
`go clean github.com/holochain/holochain-proto`
