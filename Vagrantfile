# -*- mode: ruby -*-
# vi: set ft=ruby :

# All Vagrant configuration is done below. The "2" in Vagrant.configure
# configures the configuration version (we support older styles for
# backwards compatibility). Please don't change it unless you know what
# you're doing.
Vagrant.configure("2") do |config|
  # The most common configuration options are documented and commented below.
  # For a complete reference, please see the online documentation at
  # https://docs.vagrantup.com.

  # Every Vagrant development environment requires a box. You can search for
  # boxes at https://vagrantcloud.com/search.
  # config.vm.box = "holochain-vagrant"
  # config.vm.box_url = "https://holochain-vagrant-builds.s3-us-west-2.amazonaws.com/holochain-vagrant.zip"
  config.vm.box = "nixos/nixos-18.03-x86_64"

  # Disable automatic box update checking. If you disable this, then
  # boxes will only be checked for updates when the user runs
  # `vagrant box outdated`. This is not recommended.
  # config.vm.box_check_update = false

  # Create a forwarded port mapping which allows access to a specific port
  # within the machine from a port on the host machine. In the example below,
  # accessing "localhost:8080" will access port 80 on the guest machine.
  # NOTE: This will enable public access to the opened port
  # config.vm.network "forwarded_port", guest: 80, host: 8080

  # Create a forwarded port mapping which allows access to a specific port
  # within the machine from a port on the host machine and only allow access
  # via 127.0.0.1 to disable public access
  # config.vm.network "forwarded_port", guest: 80, host: 8080, host_ip: "127.0.0.1"

  # Create a private network, which allows host-only access to the machine
  # using a specific IP.
  # config.vm.network "private_network", ip: "192.168.33.10"

  # Create a public network, which generally matched to bridged network.
  # Bridged networks make the machine appear as another physical device on
  # your network.
  # config.vm.network "public_network"

  # Share an additional folder to the guest VM. The first argument is
  # the path on the host to the actual folder. The second argument is
  # the path on the guest to mount the folder. And the optional third
  # argument is a set of non-required options.
  # config.vm.synced_folder "../data", "/vagrant_data"

  # https://github.com/rust-lang/cargo/issues/2808
  config.vm.synced_folder ".", "/vagrant", type: "rsync", rsync__exclude: [".git/", "node_modules", "target", ".cargo", "Cargo.lock"], rsync__verbose: true

  # Provider-specific configuration so you can fine-tune various
  # backing providers for Vagrant. These expose provider-specific options.
  # Example for VirtualBox:
  #
  config.vm.provider "virtualbox" do |vb|
  #   # Display the VirtualBox GUI when booting the machine
    vb.gui = false
  #
    # Customize the amount of memory on the VM:
    vb.memory = "10000"
    vb.cpus = "4"
    vb.customize ["modifyvm", :id, "--hwvirtex", "off"]
  end
  #
  # View the documentation for the provider you are using for more
  # information on available options.

  # vagrant plugin install vagrant-nixos-plugin
  # config.vm.provision :shell, inline: "fallocate -l 4G /swapfile && chmod 0600 /swapfile && mkswap /swapfile && swapon /swapfile"

  # add some simple dev tools
  config.vm.provision :nixos,
    run: 'always',
    expression: {
      swapDevices: [ { device: "/swapfile", size: 10000 } ],
      environment: {
        systemPackages: [ :htop, :dos2unix, :vim ]
      }
    }


  # https://askubuntu.com/questions/317338/how-can-i-increase-disk-size-on-a-vagrant-vm
  # config.disksize.size = '50GB'

end
