let
  base = {

    # the commit from `develop` branch that the release is targetting
    # the final release(s) will differ from this due to changelog updates etc.
    commit = "ae0c88e3c183eb55220009cfb75056c415ac852d";

    # current documentation for the release process
    process-url = "https://hackmd.io/oWIM8H4UQQSdJMaAW4uaMg";

    core = {
     version = {
      previous = "0.0.17-alpha1";
      current = "0.0.17-alpha2";
     };
    };

    node-conductor = {
     version = {
      previous = "0.4.16-alpha1";
      current = "0.4.16-alpha2";
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
