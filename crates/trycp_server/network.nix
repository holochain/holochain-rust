{
  # Name of our deployment
  network.description = "TrycpServer";

  # It consists of a single server named 'trycpserver'
  trycpserver =
    # Every server gets passed a few arguments, including a reference
    # to nixpkgs (pkgs)
    { config, pkgs, ... }:
    let
      # We import our custom packages from ./default passing pkgs as argument
      packages = import ./default.nix { pkgs = pkgs; };
      # And this is the application we'd like to deploy
      trycp      = packages.trycp;
    in
    {
      # We'll be running our application on port 9000

#      networking.firewall.enable = true;
      # We will open up port 22 (SSH) as well otherwise we're locking ourselves out
#      networking.firewall.allowedTCPPorts = [ 80 8080 22 ];
#      networking.firewall.allowPing = true;

      # Port forwarding using iptables
#      networking.firewall.extraCommands = ''
#        iptables -t nat -A PREROUTING -p tcp --dport 80 -j REDIRECT --to-port 8080
#      '';

      # To run trycp_server we're going to use a systemd service
      # We can configure the service to automatically start on boot and to restart
      # the process in case it crashes
      systemd.services.trycpserver = {
        description = "trycp";
        # Start the service after the network is available
        after = [ "network.target" ];
        wantedBy = [ "multi-user.target" ];
        # We're going to run it on port 9000 in production
        environment = { PORT = "9000"; };
        serviceConfig = {
          # The actual command to run
          ExecStart = "${trycp}/trycp_server";
          # For security reasons we'll run this process as a special 'trycp' user
          User = "trycp";
          Restart = "always";
        };
      };

      # And lastly we ensure the user we run our project as is created
      users.extraUsers = {
        trycp = { };
      };
    };
}
