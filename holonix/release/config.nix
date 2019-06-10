let
  base = {

    # the commit from `develop` branch that the release is targetting
    # the final release(s) will differ from this due to changelog updates etc.
    commit = "30e166a5793ab8a80858f59e72a4fb4af5654f42";

    # current documentation for the release process
    process-url = "https://hackmd.io/LTG8XfU4Q_6VB98tXz8Gag";

    core = {
     version = {
      previous = "0.0.18-alpha1";
      current = "0.0.19-alpha1";
     };
    };

    node-conductor = {
     version = {
      previous = "0.4.17-alpha1";
      current = "0.4.18-alpha1";
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
