rec {
 # the commit from `develop` branch that the release is targetting
 # the final release(s) will differ from this due to changelog updates etc.
 commit = "8a4916b0a40adf5cb5269f9cc2fbb6e0d9087aa8";

 # current documentation for the release process
 process-url = "https://hackmd.io/jRoqPP-NSly7SimnFb1_0Q";

 version = {
  previous = "0.0.22-alpha1";
  current = "0.0.23-alpha1";
 };

 tag = "v${version.current}";
 branch = "release-${version.current}";
}
