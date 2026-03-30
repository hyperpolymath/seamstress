# TEST-NEEDS: seamstress

## Current State

| Category | Count | Details |
|----------|-------|---------|
| **Source modules** | 13 | Rust seamctl (7: cli, lib, main, model, graph, report, validate) + Elixir seamstressd (2: application, runner) + config |
| **Unit tests** | 2 | seamstressd_test.exs (2 assertions), no Rust tests at all |
| **Integration tests** | 0 | None |
| **E2E tests** | 0 | None |
| **Benchmarks** | 0 | None |
| **Fuzz tests** | 0 | placeholder.txt only |

## What's Missing

### P2P Tests (CRITICAL)
- [ ] No tests for seamctl <-> seamstressd communication
- [ ] No tests for the graph module's dependency resolution

### E2E Tests (CRITICAL)
- [ ] No test running seamctl CLI commands
- [ ] No test running seamstressd as a service

### Aspect Tests
- [ ] **Security**: No input validation tests for seamctl
- [ ] **Performance**: No tests for graph resolution at scale
- [ ] **Concurrency**: No concurrent operation tests
- [ ] **Error handling**: No tests for malformed input, missing dependencies

### Build & Execution
- [ ] Rust seamctl has 0 tests -- not even a compilation test
- [ ] No Elixir integration test

### Benchmarks Needed
- [ ] Graph resolution performance
- [ ] Report generation throughput

### Self-Tests
- [ ] No healthcheck endpoint for seamstressd
- [ ] No self-test mode for seamctl

## FLAGGED ISSUES
- **7 Rust source files with ZERO tests** -- completely untested CLI tool
- **Elixir daemon with 2 test assertions** -- effectively untested service
- **fuzz/placeholder.txt** -- fake fuzz testing claim

## Priority: P0 (CRITICAL)
