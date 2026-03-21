use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use petgraph::graph::DiGraph;
use serde_json::json;
use std::collections::HashMap;
use walkdir::WalkDir;

use crate::model::SeamRecord;

fn default_root() -> Utf8PathBuf {
    Utf8PathBuf::from(".")
}

fn discover(root: &Utf8PathBuf) -> Vec<Utf8PathBuf> {
    let mut files = vec![];
    for e in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let p = e.path();
        if !p.is_file() {
            continue;
        }
        let s = p.to_string_lossy();
        let is_candidate_dir = s.contains("/seams/records/") || (s.contains("/targets/") && s.contains("/seams/records/"));
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

pub fn run(root: Option<&str>, format: &str, out: Option<&str>) -> Result<()> {
    let root = root.map(Utf8PathBuf::from).unwrap_or_else(default_root);
    let files = discover(&root);
    if files.is_empty() {
        return Err(anyhow!("no seam records found"));
    }

    let mut g = DiGraph::<String, String>::new();
    let mut node_map: HashMap<String, petgraph::graph::NodeIndex> = HashMap::new();
    let mut seams = vec![];

    for f in files {
        let raw = std::fs::read_to_string(&f).with_context(|| format!("read {f}"))?;
        let rec: SeamRecord = serde_json::from_str(&raw).with_context(|| format!("decode {f}"))?;

        let a_idx = *node_map.entry(rec.side_a.clone()).or_insert_with(|| g.add_node(rec.side_a.clone()));
        let b_idx = *node_map.entry(rec.side_b.clone()).or_insert_with(|| g.add_node(rec.side_b.clone()));
        g.add_edge(a_idx, b_idx, rec.id.clone());
        seams.push(rec.id);
    }

    let components: Vec<&String> = g.node_weights().collect();
    let edges: Vec<serde_json::Value> = g.edge_indices().map(|ei| {
        let (a, b) = g.edge_endpoints(ei).unwrap();
        let id = &g[ei];
        json!({"from": g[a], "to": g[b], "seam": id})
    }).collect();

    let payload = json!({
        "components": components,
        "edges": edges,
        "seams": seams,
    });

    let rendered = if format == "json" {
        serde_json::to_string_pretty(&payload)?
    } else if format == "dot" {
        let mut s = String::from("digraph seams {\n");
        for ei in g.edge_indices() {
            let (a, b) = g.edge_endpoints(ei).unwrap();
            let id = &g[ei];
            s.push_str(&format!("  \"{}\" -> \"{}\" [label=\"{}\"];\n", g[a], g[b], id));
        }
        s.push_str("}\n");
        s
    } else {
        return Err(anyhow!("unsupported format: {format} (use json|dot)"));
    };

    if let Some(out) = out {
        let outp = Utf8PathBuf::from(out);
        if let Some(parent) = outp.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&outp, rendered)?;
        println!("Wrote {outp}");
    } else {
        println!("{rendered}");
    }

    Ok(())
}
