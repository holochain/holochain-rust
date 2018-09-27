# 11. Build HCadmin app with Qt/C++

Date: 2018-05-16

## Status

Accepted

## Context

In the alpha version we had *hcd* that runs a single Holochain app as a background daemon.

Going forward we will need a more sophisticated administration app for several reasons:
* UX - One end-user friendly tool with GUI to administer all installed and running genomes.
* Coalescing of the network layer, i.e. having one network manager that handles ports for example.
* In the mid-term critical path of having a generalized Holochain UI / browser (i.e. developments made for the new admin app would serve the Holochain browser app).

Since with ADR #8 we want to go mobile first, we need to have an easy way to run dApps and hApps on mobile phones.

Qt is cross-plateform and sports QML for rapid UI development. It compiles the same code natively to Windows/Linux/MacOS/Android/iOS/Blackberry.

For system and network capabilities, C++ would be at our disposal.

## Decision

Use the Qt framework for building a cross-platform app that replaces *hcadmin* for genome management & deployment, and possibly integrating *hcd* into it if we can manage the Rust integration.

## Consequences

* We could have a system tray user interface to manage/start/stop genomes.
* We could have the exact same UI on all relevant platforms including mobile with only one code base.
* A mobile hApp would be easy to build.
* Cross-platform C++ code needs to be maintained.
* UI/UX needs to be considered regarding conforming to different platform standards vs. creating one Holochain design & feel.  
* We need to figure out how to incorporate the rust based binary.
