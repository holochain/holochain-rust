let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;
  release = import ../../config.nix;
  git = import ../../../git/config.nix;

  name = "hc-release-github-pr";

  # a few things should already be done by this point so precheck them :)
  template =
  ''
Release ${release.core.version.current}

current release process: ${release.process-url}

## Preparation

First checkout `develop` and `git pull` to ensure you are up to date locally.
Then run `nix-shell --run hc-prepare-release`

- [x] develop is green
- [x] correct dev pulse commit + version + url hash
- [x] correct core version
- [x] correct node conductor
- [x] correct release process url

## PR into master

- [ ] reviewed and updated CHANGELOG
- [ ] release PR merged into `master`

## Build and deploy release artifacts

- [ ] release cut from `master` with `hc-do-release`
- [ ] core release tag + linux/mac/windows artifacts on github
  - travis build: {{ build url }}
  - artifacts: https://github.com/holochain/holochain-rust/releases/tag/${release.core.tag}
- [ ] node release tag + linux/mac/windows artifacts on github
  - travis build: {{ build url }}
  - artifacts: https://github.com/holochain/holochain-rust/releases/tag/${release.node-conductor.tag}
- [ ] all release artifacts found by `hc-check-release-artifacts`
- [ ] npmjs deploy with `hc-release-npm-deploy` then `hc-release-npm-check-version`
- [ ] `unknown` release assets renamed to `ubuntu`

## PR into develop

- [ ] `hc-release-merge-back`
- [ ] `develop` PR changelog cleaned up
  - [ ] no new items from `develop` under recently released changelog header
- [ ] merge `develop` PR

## Finalise

- [ ] dev pulse is live on medium
- [ ] `hc-release-pulse-sync`

'';

  script = pkgs.writeShellScriptBin name
  ''
   echo
   echo 'ensure github PR'
   git config --local hub.upstream ${git.github.repo}
   git config --local hub.forkrepo ${git.github.repo}
   git config --local hub.forkremote ${git.github.upstream}
   if [ "$(git rev-parse --abbrev-ref HEAD)" == "${release.branch}" ]
    then
     git add . && git commit -am 'Release ${release.core.version.current}'
     git push && git hub pull new -b 'master' -m '${template}' --no-triangular ${release.branch}
    else
     echo "current branch is not ${release.branch}!"
     exit 1
   fi
  '';
in
script
