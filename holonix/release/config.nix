let
  base = {

    # the commit from `develop` branch that the release is targetting
    # the final release(s) will differ from this due to changelog updates etc.
    commit = "04e4bbd8ea0dc187e0d4c0960ac32841a4493645";

    # current documentation for the release process
    process-url = "https://hackmd.io/8kfuapVEQSuWSMiuI0LYhg";

    core = {
     version = {
      previous = "0.0.13-alpha1";
      current = "0.0.14-alpha1";
     };
    };

    node-conductor = {
     version = {
      previous = "0.4.12-alpha1";
      current = "0.4.13-alpha1";
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
