let
  base = {

    # the commit from `develop` branch that the release is targetting
    # the final release(s) will differ from this due to changelog updates etc.
    commit = "c065bbdbc4af29ac7f837efb1531a6011b60f86d";

    # current documentation for the release process
    process-url = "https://hackmd.io/oWIM8H4UQQSdJMaAW4uaMg";

    core = {
     version = {
      previous = "0.0.15-alpha1";
      current = "0.0.16-alpha1";
     };
    };

    node-conductor = {
     version = {
      previous = "0.4.14-alpha1";
      current = "0.4.15-alpha1";
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
