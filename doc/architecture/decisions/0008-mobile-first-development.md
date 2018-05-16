# 8. Mobile first development

Date: 2018-05-16

## Status

Accepted

## Context

Mobile is the main platform today. We want to reach as much users as possible.

## Decision

Support only 64bits platforms and not any 32bits platforms because we may have issues with cryptography on 32bits.

## Consequences

Have mobile testing in place and always test on mobile before releasing.
Development has mobile platform performance considerations.
Holo binding helps achieve this in a short-term way.
