use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use walkdir::WalkDir;

use crate::model::SeamRecord;

fn default_root() -> Utf8PathBuf {
    Utf8PathBuf::from(".")
}

fn default_schema_path() -> Utf8PathBuf {
    Utf8PathBuf::from("seams/schema/seam-record.schema.json")
}

fn discover_seam_files(root: &Utf8PathBuf) -> Vec<Utf8PathBuf> {
    let mut files = vec![];
    for e in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let p = e.path();
        if !p.is_file() {
            continue;
        }
        let s = p.to_string_lossy();
        // Search common roots; keep simple and robust in v0.
        let is_candidate_dir = s.contains("/seams/records/") || s.contains("/targets/") && s.contains("/seams/records/");
        if !is_candidate_dir {
            continue;
        }
        if s.ends_with(".seam.json") {
            if let Ok(up) = Utf8PathBuf::from_path_buf(p.to_path_buf()) {
                files.push(up);
            }
        }
    }
    files.sort();
    files
}

fn load_json(path: &Utf8PathBuf) -> Result<Value> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {path}"))?;
    let v: Value = serde_json::from_str(&raw).with_context(|| format!("parse json {path}"))?;
    Ok(v)
}

fn compile_schema(schema_path: &Utf8PathBuf) -> Result<JSONSchema> {
    let schema_json = load_json(schema_path)?;
    JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema_json)
        .map_err(|e| anyhow!("schema compile failed: {e}"))
}

fn policy_checks(record: &SeamRecord, record_path: &Utf8PathBuf, root: &Utf8PathBuf) -> Vec<String> {
    let mut errs = vec![];
    if record.owners.is_empty() {
        errs.push("owners must be non-empty".into());
    }
    if record.compat_policy.rules.is_empty() {
        errs.push("compat_policy.rules must be non-empty".into());
    }
    if record.compat_policy.deprecation_window_days < 1 {
        errs.push("compat_policy.deprecation_window_days must be >= 1".into());
    }
    // Ensure checklist statuses exist and are non-empty
    for (name, st) in [
        ("checks.conformance", &record.checks.conformance.status),
        ("checks.no_hidden_channels", &record.checks.no_hidden_channels.status),
        ("checks.evolution", &record.checks.evolution.status),
    ] {
        if st.trim().is_empty() {
            errs.push(format!("{name}.status must be non-empty"));
        }
    }

    // referenced files exist (contract_artifacts + vectors paths)
    for a in &record.contract_artifacts {
        let p = root.join(&a.path);
        if !p.exists() {
            errs.push(format!("missing contract_artifact path: {} (from {})", a.path, record_path));
        }
    }
    for tv in &record.test_vectors {
        let p = root.join(tv);
        if !p.exists() {
            errs.push(format!("missing test_vectors path: {} (from {})", tv, record_path));
        }
    }

    errs
}

pub fn run(root: Option<&str>, schema: Option<&str>) -> Result<()> {
    let root = root.map(Utf8PathBuf::from).unwrap_or_else(default_root);
    let schema_path = schema.map(Utf8PathBuf::from).unwrap_or_else(default_schema_path);

    let compiled = compile_schema(&schema_path)?;
    let seam_files = discover_seam_files(&root);

    if seam_files.is_empty() {
        return Err(anyhow!("no *.seam.json files found under seams/records or targets/**/seams/records"));
    }

    let mut schema_errors = vec![];
    let mut policy_errors = vec![];

    for f in seam_files {
        let v = load_json(&f)?;
        if let Err(errs) = compiled.validate(&v) {
            for e in errs {
                schema_errors.push(format!("{f}: {e}"));
            }
            continue;
        }
        // Deserialize to run policy checks
        let rec: SeamRecord = serde_json::from_value(v).with_context(|| format!("decode seam record {f}"))?;
        for pe in policy_checks(&rec, &f, &root) {
            policy_errors.push(format!("{f}: {pe}"));
        }
    }

    if !schema_errors.is_empty() {
        eprintln!("Schema validation errors:");
        for e in &schema_errors {
            eprintln!("- {e}");
        }
        std::process::exit(2);
    }

    if !policy_errors.is_empty() {
        eprintln!("Policy errors:");
        for e in &policy_errors {
            eprintln!("- {e}");
        }
        std::process::exit(3);
    }

    println!("OK: seam records validated");
    Ok(())
}
