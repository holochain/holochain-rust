rec {
 # the commit from `develop` branch that the release is targetting
 # the final release(s) will differ from this due to changelog updates etc.
 commit = "9b482b94b37b2b82e07700de6b2f73a63edb0a5f";

 # current documentation for the release process
 process-url = "https://hackmd.io/LTG8XfU4Q_6VB98tXz8Gag";

 version = {
  previous = "0.0.20-alpha3";
  current = "0.0.21-alpha1";
 };

 tag = "v${version.current}";
 branch = "release-${version.current}";
}
