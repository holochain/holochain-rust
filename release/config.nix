rec {
 # the commit from `develop` branch that the release is targetting
 # the final release(s) will differ from this due to changelog updates etc.
 commit = "fd3f399fbae58f54d96b20e9122765534895c810";

 # current documentation for the release process
 process-url = "https://hackmd.io/jRoqPP-NSly7SimnFb1_0Q";

 version = {
  previous = "0.0.21-alpha1";
  current = "0.0.22-alpha1";
 };

 tag = "v${version.current}";
 branch = "release-${version.current}";
}
