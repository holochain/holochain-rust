# 8. Mobile first development

Date: 2018-05-16

## Status

Accepted

## Context

Mobile is the main platform today. We want to reach as many users as possible.
Mobile use has two big constraints:  battery life (and consequent sleeping of apps), and bandwidth because of costs
Go platform development ignored mobile from the start and we found out late about the problems of compiling to mobile.

## Decision

Target a mobile build from the start.
Do not initially worry about battery/bandwidth constraints, assuming that ADR 0006 will handle solve this issue in the medium term, and that advance in technology will handle it in the long-term.

## Consequences

Have mobile testing in place and always test on mobile before releasing.
Development has mobile platform performance considerations.
Holo binding helps achieve this in a short-term way.
