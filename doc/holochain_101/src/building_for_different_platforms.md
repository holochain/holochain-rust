# Building for Different Platforms

Holochain apps aren't limited to just running on laptop and desktop computers, they can be configured to run on mostly any device with a CPU, from Raspberry Pis to Android smartphones.

While the tools to support building for these various platforms haven't been fully developed yet, it is still important for developers to note that being written in Rust, Holochain has this affordance.

So far, by utilizing bindings for the C language with Holochain, a cross-platform tool for starting and stopping app instances has been developed, called [HoloSqape](https://github.com/holochain/holosqape). This particular implementation targets Ubuntu (and Linux), MacOS, and Windows. Note that if you are looking for a good language to develop a cross-platform GUI in, [Qt and Qml](https://doc.qt.io/qt-5.11/qtqml-index.html) which is utilized by HoloSqape is a good option.

Such a tool, that can load, start, and stop a Holochain app instance (or apps), is called a "Container" in Holochain terminology.

Another approach to running Holochain apps on different platforms,would be to include Holochain itself in your native application, whether it be an Electron app, or an Android app.

There has been some work done to explore building Holochain for Android. If the technical details of this interest you, see [this article](./building_for_android.md)

Now that you know what's possible in terms of platform options, carry on with getting to know Holochain app development!

