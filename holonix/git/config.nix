let
 base = {

  github = {
    user = "holochain";
    repo-name = "holochain-rust";
    upstream = "origin";
  };

 };

 derived = {

  github = base.github // {
   repo = "${base.github.user}/${base.github.repo-name}";
  };

 };

in
base // derived
