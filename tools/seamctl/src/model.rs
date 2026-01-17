use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SeamRecord {
    pub id: String,
    pub title: String,
    pub status: String,
    pub owners: Vec<Owner>,
    pub side_a: String,
    pub side_b: String,
    pub boundary_type: String,
    pub data_flows: Vec<DataFlow>,
    pub contract_artifacts: Vec<ContractArtifact>,
    pub compat_policy: CompatPolicy,
    pub semantic_invariants: Vec<String>,
    pub security_invariants: Vec<String>,
    pub privacy_invariants: Vec<String>,
    pub test_vectors: Vec<String>,
    pub checks: Checks,
    pub slo: Slo,
    pub failure_behavior: FailureBehavior,
    pub observability: Observability,
    pub change_process: ChangeProcess,
    pub rollout_backout: RolloutBackout,
}

#[derive(Debug, Deserialize)]
pub struct Owner {
    pub name: String,
    pub contact: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct DataFlow {
    pub name: String,
    pub direction: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct ContractArtifact {
    pub path: String,
    pub kind: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct CompatPolicy {
    pub strategy: String,
    pub rules: Vec<String>,
    pub deprecation_window_days: i64,
}

#[derive(Debug, Deserialize)]
pub struct Checks {
    pub conformance: CheckStatus,
    pub no_hidden_channels: CheckStatus,
    pub evolution: CheckStatus,
}

#[derive(Debug, Deserialize)]
pub struct CheckStatus {
    pub status: String,
    pub notes: String,
}

#[derive(Debug, Deserialize)]
pub struct Slo {
    pub latency_ms_p95: i64,
    pub error_rate_max: f64,
}

#[derive(Debug, Deserialize)]
pub struct FailureBehavior {
    pub timeouts: String,
    pub retries: String,
    pub backpressure: String,
    pub idempotency: String,
}

#[derive(Debug, Deserialize)]
pub struct Observability {
    pub metrics: Vec<String>,
    pub logs: Logs,
    pub tracing: Tracing,
}

#[derive(Debug, Deserialize)]
pub struct Logs {
    pub required_fields: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Tracing {
    pub propagation: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangeProcess {
    pub required_reviewers: Vec<String>,
    pub gates: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RolloutBackout {
    pub rollout_steps: Vec<String>,
    pub backout_steps: Vec<String>,
}
