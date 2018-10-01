NOTE: This page is out of date.  TODO

# Introduction
This document addresses the fundamental areas of security:

**Dominant Security Issues**

**Key Management and Key Evolution**
* DPKI Holochain app

**Built in Currencies**
* DOS remediation
* Neighborhoods blacklisting bad actors
* When to use the blacklist tools
* Standard Process to declare yourself into a holochain

**Members and Blacklisting**
* Part of our stdlib will have blacklisting
* Bloomfilter not just on msgIDs, but on sourceIDs**
* If you're not a valid source, we drop your packets
* Enomomy of resources - 

**Transit Security**
* Wire protocols
* MITM and replay attacks
* Gossip Protocols

**Localhost Security**
* Privilege Escalation

# Discussion of Common Security practices

## Relevant Security Issues

**Reputation currency (add to glossary)**

* DDOS
* MITM

## Obviated Security Issues

* Network layer security

# Discussion - needs integration into document

non-consensus system - RAFT is useless
providing provenance for imprecise timed of a thing from a place
DHT's nodes don't have the issues of consensus.  They use a verification of agreements... not consensus... it's just validation, not time-bound.

