// SPDX-License-Identifier: PMPL-1.0-or-later
// Integration tests for seamctl.
//
// These tests exercise the public API surface of the library crates without
// invoking the binary directly, so they run under `cargo test` with no
// additional tooling.  The tests cover:
//
//   - validate::run  — schema validation and policy checks
//   - graph::run     — dependency graph construction
//   - report::run    — AsciiDoc report generation
//   - model          — SeamRecord deserialization
//
// Each test creates an isolated temporary workspace so tests can run in
// parallel without filesystem contention.

use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns a fresh temporary directory unique to this test invocation.
fn tmp_dir(label: &str) -> PathBuf {
    let base = std::env::temp_dir();
    let name = format!("seamctl_test_{label}_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos());
    let path = base.join(name);
    fs::create_dir_all(&path).expect("failed to create temp dir");
    path
}

/// Write a minimal but structurally valid seam record JSON to `path`.
fn write_valid_record(dir: &Path, id: &str) {
    let records_dir = dir.join("seams").join("records");
    fs::create_dir_all(&records_dir).unwrap();

    let record = serde_json::json!({
        "id": id,
        "title": "Test Seam",
        "status": "active",
        "owners": [{"name": "Alice", "contact": "alice@example.com", "role": "owner"}],
        "side_a": "service-alpha",
        "side_b": "service-beta",
        "boundary_type": "http-api",
        "data_flows": [{"name": "requests", "direction": "a->b", "description": "REST calls"}],
        "contract_artifacts": [],
        "compat_policy": {
            "strategy": "semver",
            "rules": ["no breaking changes"],
            "deprecation_window_days": 90
        },
        "semantic_invariants": ["idempotent"],
        "security_invariants": ["mTLS required"],
        "privacy_invariants": ["no PII in URL"],
        "test_vectors": [],
        "checks": {
            "conformance": {"status": "pass", "notes": "verified"},
            "no_hidden_channels": {"status": "done", "notes": "audited"},
            "evolution": {"status": "pass", "notes": ""}
        },
        "slo": {"latency_ms_p95": 200, "error_rate_max": 0.005},
        "failure_behavior": {
            "timeouts": "10s",
            "retries": "3 with exponential back-off",
            "backpressure": "token bucket",
            "idempotency": "request-id header"
        },
        "observability": {
            "metrics": ["requests_total", "error_rate"],
            "logs": {"required_fields": ["trace_id", "span_id"]},
            "tracing": {"propagation": ["W3C TraceContext"]}
        },
        "change_process": {
            "required_reviewers": ["seam-owners"],
            "gates": ["ci-pass", "schema-validated"]
        },
        "rollout_backout": {
            "rollout_steps": ["blue-green deploy"],
            "backout_steps": ["rollback via feature flag"]
        }
    });

    let path = records_dir.join(format!("{id}.seam.json"));
    fs::write(path, serde_json::to_string_pretty(&record).unwrap()).unwrap();
}

/// Write the minimal permissive JSON Schema that seamctl compiles.
fn write_permissive_schema(dir: &Path) {
    let schema_dir = dir.join("seams").join("schema");
    fs::create_dir_all(&schema_dir).unwrap();

    let schema = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object"
    });

    let path = schema_dir.join("seam-record.schema.json");
    fs::write(path, serde_json::to_string(&schema).unwrap()).unwrap();
}

// ---------------------------------------------------------------------------
// validate::run tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod validate_tests {
    use super::*;
    use seamctl::validate;

    /// An empty directory (no schema, no records) must return an error.
    #[test]
    fn empty_dir_returns_error() {
        let root = tmp_dir("validate_empty");
        let result = validate::run(Some(root.to_str().unwrap()), None);
        assert!(result.is_err(), "expected Err for empty directory");
    }

    /// Schema present but no records — must still return an error.
    #[test]
    fn schema_only_no_records_returns_error() {
        let root = tmp_dir("validate_schema_only");
        write_permissive_schema(&root);
        let result = validate::run(Some(root.to_str().unwrap()), None);
        assert!(result.is_err(), "expected Err when no seam records exist");
    }

    /// A valid record with a permissive schema should succeed.
    #[test]
    fn valid_record_passes_validation() {
        let root = tmp_dir("validate_valid");
        write_permissive_schema(&root);
        write_valid_record(&root, "seam-valid-001");

        let result = validate::run(Some(root.to_str().unwrap()), None);
        assert!(result.is_ok(), "expected Ok for valid record: {result:?}");
    }

    /// A non-existent root path must return an error (cannot read schema).
    #[test]
    fn nonexistent_root_returns_error() {
        let result = validate::run(Some("/tmp/seamctl_nonexistent_path_abc123xyz"), None);
        assert!(result.is_err(), "expected Err for nonexistent root");
    }

    /// Multiple valid records in the same workspace should all pass.
    #[test]
    fn multiple_valid_records_pass() {
        let root = tmp_dir("validate_multi");
        write_permissive_schema(&root);
        write_valid_record(&root, "seam-multi-001");
        write_valid_record(&root, "seam-multi-002");
        write_valid_record(&root, "seam-multi-003");

        let result = validate::run(Some(root.to_str().unwrap()), None);
        assert!(result.is_ok(), "expected Ok for multiple valid records: {result:?}");
    }

    /// A record with invalid JSON should cause a parse error.
    #[test]
    fn malformed_json_returns_error() {
        let root = tmp_dir("validate_malformed");
        write_permissive_schema(&root);

        let records_dir = root.join("seams").join("records");
        fs::create_dir_all(&records_dir).unwrap();
        fs::write(records_dir.join("bad.seam.json"), b"{ not valid json }").unwrap();

        let result = validate::run(Some(root.to_str().unwrap()), None);
        assert!(result.is_err(), "expected Err for malformed JSON");
    }
}

// ---------------------------------------------------------------------------
// graph::run tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod graph_tests {
    use super::*;
    use seamctl::graph;

    /// No records must return an error (nothing to graph).
    #[test]
    fn no_records_returns_error() {
        let root = tmp_dir("graph_empty");
        let result = graph::run(Some(root.to_str().unwrap()), "json", None);
        assert!(result.is_err(), "expected Err with no seam records");
    }

    /// One valid record should produce valid JSON output.
    #[test]
    fn single_record_produces_json() {
        let root = tmp_dir("graph_single");
        write_valid_record(&root, "seam-graph-001");

        // Write output to a temp file.
        let out_path = root.join("graph.json");
        let result = graph::run(
            Some(root.to_str().unwrap()),
            "json",
            Some(out_path.to_str().unwrap()),
        );

        assert!(result.is_ok(), "expected Ok: {result:?}");
        assert!(out_path.exists(), "output file must be created");

        let content = fs::read_to_string(&out_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.get("edges").is_some(), "JSON output must have 'edges' key");
    }

    /// Unsupported format must return an error.
    #[test]
    fn unsupported_format_returns_error() {
        let root = tmp_dir("graph_bad_fmt");
        write_valid_record(&root, "seam-fmt-001");

        let result = graph::run(Some(root.to_str().unwrap()), "xml", None);
        assert!(result.is_err(), "expected Err for unsupported format");
    }
}

// ---------------------------------------------------------------------------
// report::run tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod report_tests {
    use super::*;
    use seamctl::report;

    /// No records must return an error.
    #[test]
    fn no_records_returns_error() {
        let root = tmp_dir("report_empty");
        let result = report::run(Some(root.to_str().unwrap()), None);
        assert!(result.is_err(), "expected Err with no seam records");
    }

    /// A valid record should produce an AsciiDoc report file.
    #[test]
    fn valid_record_produces_report() {
        let root = tmp_dir("report_valid");
        write_valid_record(&root, "seam-report-001");

        let out_path = root.join("seams").join("reports").join("seams-report.adoc");
        let out_str = out_path.to_str().unwrap().to_string();

        let result = report::run(Some(root.to_str().unwrap()), Some(&out_str));
        assert!(result.is_ok(), "expected Ok for valid record: {result:?}");
        assert!(out_path.exists(), "report file must be created");

        let content = fs::read_to_string(&out_path).unwrap();
        assert!(content.contains("= Seam Report"), "report must start with AsciiDoc title");
        assert!(content.contains("seam-report-001"), "report must include record ID");
    }
}

// ---------------------------------------------------------------------------
// model deserialization tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod model_tests {
    use seamctl::model::{SeamRecord, Owner};

    /// A fully specified record JSON must deserialise without error.
    #[test]
    fn full_record_deserialises() {
        let json = serde_json::json!({
            "id": "seam-model-001",
            "title": "Model Test Seam",
            "status": "active",
            "owners": [{"name": "Bob", "contact": "bob@test.invalid", "role": "tech-lead"}],
            "side_a": "frontend",
            "side_b": "backend",
            "boundary_type": "grpc",
            "data_flows": [{"name": "rpc", "direction": "a->b", "description": "gRPC call"}],
            "contract_artifacts": [],
            "compat_policy": {
                "strategy": "semver",
                "rules": ["no breaking changes in minor"],
                "deprecation_window_days": 60
            },
            "semantic_invariants": [],
            "security_invariants": [],
            "privacy_invariants": [],
            "test_vectors": [],
            "checks": {
                "conformance": {"status": "pass", "notes": ""},
                "no_hidden_channels": {"status": "pass", "notes": ""},
                "evolution": {"status": "pass", "notes": ""}
            },
            "slo": {"latency_ms_p95": 50, "error_rate_max": 0.001},
            "failure_behavior": {
                "timeouts": "3s",
                "retries": "2",
                "backpressure": "circuit breaker",
                "idempotency": "yes"
            },
            "observability": {
                "metrics": ["rpc_calls_total"],
                "logs": {"required_fields": ["request_id"]},
                "tracing": {"propagation": ["W3C"]}
            },
            "change_process": {"required_reviewers": ["cto"], "gates": ["tests-pass"]},
            "rollout_backout": {
                "rollout_steps": ["canary deploy"],
                "backout_steps": ["revert"]
            }
        });

        let record: Result<SeamRecord, _> = serde_json::from_value(json);
        assert!(record.is_ok(), "expected successful deserialisation: {record:?}");

        let r = record.unwrap();
        assert_eq!(r.id, "seam-model-001");
        assert_eq!(r.title, "Model Test Seam");
        assert_eq!(r.owners.len(), 1);
    }

    /// Missing required field must produce a deserialisation error.
    #[test]
    fn missing_required_field_fails() {
        // Omit the "owners" field, which is required.
        let json = serde_json::json!({
            "id": "seam-missing",
            "title": "Missing Fields",
            "status": "active"
            // owners, side_a, side_b, … omitted
        });

        let result: Result<SeamRecord, _> = serde_json::from_value(json);
        assert!(result.is_err(), "expected Err for incomplete record");
    }

    /// Owner struct fields must deserialise correctly.
    #[test]
    fn owner_deserialises_correctly() {
        let json = serde_json::json!({
            "name": "Carol",
            "contact": "carol@example.com",
            "role": "product-owner"
        });

        let owner: Result<Owner, _> = serde_json::from_value(json);
        assert!(owner.is_ok(), "expected Ok for well-formed owner");

        let o = owner.unwrap();
        assert_eq!(o.name, "Carol");
        assert_eq!(o.contact, "carol@example.com");
        assert_eq!(o.role, "product-owner");
    }
}
