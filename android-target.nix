with import <nixpkgs> {

  crossSystem = {
    #config = "armv7a-unknown-linux-androideabi";
    config = "aarch64-unknown-linux-android";
    sdkVer = "24";
    ndkVer = "18b";
    platform = {
       name = "aarch64-multiplatform";
       kernelMajor = "2.6"; # Using "2.6" enables 2.6 kernel syscalls in glibc.
       kernelBaseConfig = "defconfig";
       kernelArch = "arm64";
       kernelDTB = true;
       kernelAutoModules = true;
       kernelPreferBuiltin = true;
       kernelExtraConfig = ''
         # Raspberry Pi 3 stuff. Not needed for kernels >= 4.10.
         ARCH_BCM2835 y
         BCM2835_MBOX y
         BCM2835_WDT y
         RASPBERRYPI_FIRMWARE y
         RASPBERRYPI_POWER y
         SERIAL_8250_BCM2835AUX y
         SERIAL_8250_EXTENDED y
         SERIAL_8250_SHARE_IRQ y
         # Cavium ThunderX stuff.
         PCI_HOST_THUNDER_ECAM y
         # Nvidia Tegra stuff.
         PCI_TEGRA y
         # The default (=y) forces us to have the XHCI firmware available in initrd,
         # which our initrd builder can't currently do easily.
         USB_XHCI_TEGRA m
       '';
       kernelTarget = "Image";
       gcc = {
         arch = "armv8-a";
       };
     };
    #platform = {
    #    name = "armeabi-v7a";
    #    gcc = { arch = "armv7-a"; float-abi = "softfp"; fpu = "vfpv3-d16"; };
    #};
    useAndroidPrebuilt = true;
  };
};

mkShell {
  buildInputs = [  ]; # your dependencies here
}