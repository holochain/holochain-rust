let
 base = {
  # the unique hash at the end of the medium post url
  # e.g. https://medium.com/@holochain/foos-and-bars-4867d777de94
  # would be 4867d777de94
  url-hash = "4ba6cf8c2e0a";
  # current dev-pulse iteration, as seen by general public
  version = "31";
  hash-list = "https://bit.ly/2LiQuJk";
 };

 derived = base // {
  tag = "dev-pulse-${base.version}";
  url = "https://medium.com/@holochain/${base.url-hash}";
 };
in
base // derived
