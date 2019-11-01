# Building for Different Platforms

Holochain is designed to run on many different platforms.  Essentially it can run on any platform that Rust and the Rust WASM interpreter targets.  Thus Holochain DNA's will be able to run on platforms ranging from Raspberry Pis to Android smartphones once the tools have been fully developed.

We have experimented with C bindings that allowed us to run Holochain DNAs, in a [Qt and Qml](https://doc.qt.io/qt-5.11/qtqml-index.html) based cross-platform deployment, which besides running on desktop machines also [worked on Android](./building_for_android.md).

We expect other approaches to running Holochain apps on different platforms to proliferate, including compiling Holochain directly in your native application, whether it be an Electron app, a command-line Rust based app, or an Android app.
