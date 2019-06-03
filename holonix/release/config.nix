let
  base = {

    # the commit from `develop` branch that the release is targetting
    # the final release(s) will differ from this due to changelog updates etc.
    commit = "e88dcc69f6b4b12109e7a4be2634743f41bac749";

    # current documentation for the release process
    process-url = "https://hackmd.io/oWIM8H4UQQSdJMaAW4uaMg";

    core = {
     version = {
      previous = "0.0.17-alpha2";
      current = "0.0.18-alpha1";
     };
    };

    node-conductor = {
     version = {
      previous = "0.4.16-alpha2";
      current = "0.4.17-alpha1";
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
