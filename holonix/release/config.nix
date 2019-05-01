let
  base = {

    # the commit from `develop` branch that the release is targetting
    # the final release(s) will differ from this due to changelog updates etc.
    commit = "aa38cb700e4262824669f9d9b64ab00c9321e4b5";

    # current documentation for the release process
    process-url = "https://hackmd.io/8kfuapVEQSuWSMiuI0LYhg";

    core = {
     version = {
      previous = "0.0.12-alpha1";
      current = "0.0.13-alpha1";
     };
    };

    node-conductor = {
     version = {
      previous = "0.4.11-alpha1";
      current = "0.4.12-alpha1";
     };
    };

  };

  derived = {

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
