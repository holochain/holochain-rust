# Building For Android

Note: These instructions for building Holochain on Android are adapted from [here](https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-android.html).

In order to get to libraries that can be linked against when building [HoloSqape](https://github.com/holochain/holosqape) for Android, you basically just need to setup up according targets for cargo.

Given that the Android SDK is installed, here are the steps to setting things up for building:

1. Install the Android tools:

    a. Install [Android Studio](https://developer.android.com/studio/)
    b. Open Android Studio and navigate to SDK Tools:
        - MacOS: `Android Studio > Preferences > Appearance & Behaviour > Android SDK > SDK Tools`
        - Linux: `Configure (gear) >  Appearance & Behavior > System Settings > Android SDK`
    c. Check the following options for installation and click OK:
        * Android SDK Tools
        * NDK
        * CMake
        * LLDB
    d. Get a beverage of your choice (or a full meal for that matter) why you wait for the lengthy download

1. Setup ANDROID_HOME env variable:

On MacOS

```bash
export ANDROID_HOME=/Users/$USER/Library/Android/sdk
```

Linux: (assuming you used defaults when installing Android Studio)

```bash
export ANDROID_HOME=$HOME/Android/Sdk
```

2. Create standalone NDKs (the commands below put the NDK in your home dir but you can put them where you like):

```bash
export NDK_HOME=$ANDROID_HOME/ndk-bundle
cd ~
mkdir NDK
${NDK_HOME}/build/tools/make_standalone_toolchain.py --api 26 --arch arm64 --install-dir NDK/arm64
${NDK_HOME}/build/tools/make_standalone_toolchain.py --api 26 --arch arm --install-dir NDK/arm
${NDK_HOME}/build/tools/make_standalone_toolchain.py --api 26 --arch x86 --install-dir NDK/x86
```

3. Add the following lines to your ```~/.cargo/config```:

```toml
[target.aarch64-linux-android]
ar = "<your $HOME value here>/NDK/arm64/bin/aarch64-linux-android-ar"
linker = "<your $HOME value here>/NDK/arm64/bin/aarch64-linux-android-clang"

[target.armv7-linux-androideabi]
ar = "<your $HOME value here>/NDK/arm/bin/arm-linux-androideabi-ar"
linker = "<your $HOME value here>/NDK/arm/bin/arm-linux-androideabi-clang"

[target.i686-linux-android]
ar = "<your $HOME value here>/NDK/x86/bin/i686-linux-android-ar"
linker = "<your $HOME value here>/NDK/x86/bin/i686-linux-android-clang"

```
(this toml file needs absolute paths, so you need to prefix the path with your home dir).

4. Now you can add those targets to your rust installation with:

```
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android
```

Finally, should now be able to build Holochain for Android with your chosen target, e.g.:

```
cd <holochain repo>
cargo build --target armv7-linux-androideabi --release
```

**NOTE:**  there is currently a problem in that `wabt` (which we use in testing as a dev dependency) won't compile on android, and the cargo builder compiles dev dependencies even though they aren't being used in release builds.  Thus as a work around, for the cargo build command above to work, you need to manually comment out the dev dependency section in both `core/Cargo.toml` and `core_api/Cargo.toml`