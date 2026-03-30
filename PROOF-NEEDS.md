# PROOF-NEEDS.md
<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->

## Current State

- **LOC**: ~1,570
- **Languages**: Rust, Elixir, Idris2, Zig
- **Existing ABI proofs**: `src/abi/*.idr` (template-level)
- **Dangerous patterns**: None detected

## What Needs Proving

### Graph Model Correctness (tools/seamctl/src/graph.rs)
- Dependency graph construction and traversal
- Prove: graph is acyclic (DAG invariant for dependency ordering)
- Prove: topological sort produces valid execution order

### Model Validation (tools/seamctl/src/model.rs, validate.rs)
- Configuration validation
- Prove: validated configurations produce well-formed dependency graphs

### Elixir Runner (services/seamstressd/)
- Workflow runner daemon
- Prove: runner executes tasks in the order dictated by the graph

## Recommended Prover

- **Idris2** for graph properties (DAG, topological ordering)

## Priority

**LOW** — Small codebase, no dangerous patterns. Graph correctness proofs would add value but the scope is limited.
