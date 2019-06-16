let
  base = {

    # the commit from `develop` branch that the release is targetting
    # the final release(s) will differ from this due to changelog updates etc.
    commit = "e612a9b79cb43b1ba4f3a3cbebf23fa194f9a91d";

    # current documentation for the release process
    process-url = "https://hackmd.io/LTG8XfU4Q_6VB98tXz8Gag";

    version = {
     previous = "0.0.19-alpha1";
     current = "0.0.20-alpha1";
    };

  };

  derived = {

    tag = "v${base.version.current}";

    branch = "release-${base.version.current}";

  };
in
base // derived
