<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->
<!-- TOPOLOGY.md — Project architecture map and completion dashboard -->
<!-- Last updated: 2026-02-19 -->

# Seamstress — Project Topology

## System Architecture

```
                        ┌─────────────────────────────────────────┐
                        │              SYSTEM ARCHITECT           │
                        │        (Boundary Specs, Seam Audit)     │
                        └───────────────────┬─────────────────────┘
                                            │
                                            ▼
                        ┌─────────────────────────────────────────┐
                        │           SEAMSTRESS ENGINE             │
                        │                                         │
                        │  ┌───────────┐  ┌───────────────────┐  │
                        │  │ Seam      │  │  Boundary         │  │
                        │  │ Definitions│ │  Invariants       │  │
                        │  └─────┬─────┘  └────────┬──────────┘  │
                        │        │                 │              │
                        │  ┌─────▼─────┐  ┌────────▼──────────┐  │
                        │  │ Evolution │  │  Audit Tooling    │  │
                        │  │ Tracker   │  │                   │  │
                        │  └─────┬─────┘  └────────┬──────────┘  │
                        └────────│─────────────────│──────────────┘
                                 │                 │
                                 ▼                 ▼
                        ┌─────────────────────────────────────────┐
                        │           TARGET COMPONENTS             │
                        │      (Service A, Module B, API C)       │
                        └─────────────────────────────────────────┘

                        ┌─────────────────────────────────────────┐
                        │          REPO INFRASTRUCTURE            │
                        │  Justfile Automation  .machine_readable/  │
                        │  Mustfile             0-AI-MANIFEST.a2ml  │
                        └─────────────────────────────────────────┘
```

## Completion Dashboard

```
COMPONENT                          STATUS              NOTES
─────────────────────────────────  ──────────────────  ─────────────────────────────────
SEAM CORE
  Seam Definitions                  ██████░░░░  60%    Initial schema stable
  Boundary Invariants               ████░░░░░░  40%    Constraint logic refining
  Evolution Tracking                ██░░░░░░░░  20%    History stubs verified

DOCUMENTATION & TOOLS
  ARCHITECTURE.adoc                 ██████████ 100%    System design stable
  SEAMS.adoc (Index)                ████████░░  80%    Current boundaries mapped
  Audit Tooling                     ████░░░░░░  40%    CLI initial stubs

REPO INFRASTRUCTURE
  Justfile Automation               ██████████ 100%    Standard tasks verified
  .machine_readable/                ██████████ 100%    STATE tracking active
  0-AI-MANIFEST.a2ml                ██████████ 100%    AI entry point verified

─────────────────────────────────────────────────────────────────────────────
OVERALL:                            █████░░░░░  ~50%   Architecture stable, Tools pending
```

## Key Dependencies

```
Philosophy ──────► Seam Spec ──────► Boundary Check ──────► Compliance
     │                │                   │                    │
     ▼                ▼                   ▼                    ▼
Target Model ───► Evolution ──────► Audit Report ─────────► Stable Seam
```

## Update Protocol

This file is maintained by both humans and AI agents. When updating:

1. **After completing a component**: Change its bar and percentage
2. **After adding a component**: Add a new row in the appropriate section
3. **After architectural changes**: Update the ASCII diagram
4. **Date**: Update the `Last updated` comment at the top of this file

Progress bars use: `█` (filled) and `░` (empty), 10 characters wide.
Percentages: 0%, 10%, 20%, ... 100% (in 10% increments).
