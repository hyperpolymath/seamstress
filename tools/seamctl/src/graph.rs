use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use petgraph::graphmap::DiGraphMap;
use serde_json::json;
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

    let mut g: DiGraphMap<String, String> = DiGraphMap::new(); // edge weight = seam id
    let mut seams = vec![];

    for f in files {
        let raw = std::fs::read_to_string(&f).with_context(|| format!("read {f}"))?;
        let rec: SeamRecord = serde_json::from_str(&raw).with_context(|| format!("decode {f}"))?;
        g.add_node(rec.side_a.clone());
        g.add_node(rec.side_b.clone());
        g.add_edge(rec.side_a.clone(), rec.side_b.clone(), rec.id.clone());
        seams.push(rec.id);
    }

    // Cycle detection (simple): if any node reachable to itself via DFS in a small graph, flag.
    // In v0: warnings only.
    let mut warnings = vec![];
    for n in g.nodes() {
        // naive: if there's a path back, petgraph doesn't give easy on graphmap; skip deep analysis v0.
        let _ = n;
    }

    if !warnings.is_empty() {
        eprintln!("Warnings:");
        for w in warnings {
            eprintln!("- {w}");
        }
    }

    let payload = json!({
        "components": g.nodes().collect::<Vec<_>>(),
        "edges": g.all_edges().map(|(a,b,id)| json!({"from":a,"to":b,"seam":id})).collect::<Vec<_>>(),
        "seams": seams,
    });

    let rendered = if format == "json" {
        serde_json::to_string_pretty(&payload)?
    } else if format == "dot" {
        // Minimal DOT emission
        let mut s = String::from("digraph seams {\n");
        for (a, b, id) in g.all_edges() {
            s.push_str(&format!("  \"{}\" -> \"{}\" [label=\"{}\"];\n", a, b, id));
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
