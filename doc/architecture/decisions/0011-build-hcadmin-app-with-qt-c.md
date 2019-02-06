# 11. Build HCadmin app with Qt/C++

Date: 2018-05-16

## Status

proposed

## Context

In the alpha version we had *hcd* that runs a single Holochain app/agent as a background demon.

Going forward we need a more sophisticated conductor of several Holochain apps for several reasons:
* UX - One end-user friendly tool with GUI to administer all installed and running Holochain apps
* coalescing of the network layer / having one network manager, to handle ports etc.
* Mid-term roadmap entails having a generalized Holochain UI / browser that would act as an app conductor as well

Since with ADR 8 we want to go mobile first, we need to have an easy way to run Holochain apps on mobile phones.

Qt sports QML for rapid UI development and compiles the same code natively to Windows/Linux/MacOS/Android/iOS/Blackberry.
For system and network capabilities, C++ would be at our disposal.

## Decision

Use Qt framework for building a cross platform Holochain app conductor as a replacement for *hcadmin* for Holochain app management & deployment, and possiblly integrating *hcd* into it if we can manage the Rust integration.

## Consequences

* We could have a system tray app to manage/start/stop HC apps.
* We could have the exact same UI on all relevant platforms including mobile with only one code base.
* -> a mobile app would be easy to build.
* Cross-platform C++ code needs to be maintained.
* UI/UX needs to be considered regarding conforming to different platform standards vs. creating one HC design & feel.
* We need to figure out how to incorporate the rust based binary.
