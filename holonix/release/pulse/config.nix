let
 base = {
  # the unique hash at the end of the medium post url
  # e.g. https://medium.com/@holochain/foos-and-bars-4867d777de94
  # would be 4867d777de94
  url-hash = "58e5caf1916";
  # current dev-pulse iteration, as seen by general public
  version = "28";
  hash-list = "https://bit.ly/2LiQuJk";
 };

 derived = base // {
  tag = "dev-pulse-${base.version}";
  url = "https://medium.com/@holochain/${base.url-hash}";
 };
in
base // derived
