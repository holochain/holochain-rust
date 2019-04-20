let
  base = {

    # current documentation for the release process
    process-url = "https://hackmd.io/pt72afqYTWat7cuNqpAFjw";

    pulse = {
     # the unique hash at the end of the medium post url
     # e.g. https://medium.com/@holochain/foos-and-bars-4867d777de94
     # would be 4867d777de94
     url-hash = "d387ffcfac72";
     # current dev-pulse iteration, as seen by general public
     version = "24";
     # the commit from `develop` branch that the dev pulse is targetting
     # the final release(s) will differ from this due to changelog updates etc.
     commit = "494c21b9dc7927b7b171533cc20c4d39bd92b45c";
    };

    core = {
     version = {
      previous = "0.0.10-alpha2";
      current = "0.0.11-alpha1";
     };
    };

    node-conductor = {
     version = {
      previous = "0.4.9-alpha2";
      current = "0.4.10-alpha1";
     };
    };

  };

  derived = {
    pulse = base.pulse // {
     tag = "dev-pulse-${base.pulse.version}";
     url = "https://medium.com/@holochain/${base.pulse.url-hash}";
    };

    core = base.core // {
     tag = "v${base.core.version.current}";
    };

    node-conductor = base.node-conductor // {
     tag = "holochain-nodejs-v${base.node-conductor.version.current}";
    };

    branch = "release-${base.core.version.current}";

  };
in
base // derived
