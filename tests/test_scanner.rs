use std::fs;
use tempfile::tempdir;
use dev_scrub::scanner::{ArtifactType, DevScanner};

#[test]
fn test_scanner_detects_artifacts() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    // Create a mock Rust project: project_rust/target/debug/test.bin
    let rust_target = root.join("project_rust").join("target").join("debug");
    fs::create_dir_all(&rust_target).unwrap();
    fs::write(rust_target.join("test.bin"), vec![0u8; 1024]).unwrap();

    // Create a mock Node project: project_node/node_modules/lodash/index.js
    let node_modules = root.join("project_node").join("node_modules").join("lodash");
    fs::create_dir_all(&node_modules).unwrap();
    fs::write(node_modules.join("index.js"), "module.exports = {};").unwrap();

    // Create a mock .git directory with fake objects (MUST BE SKIPPED)
    let git_dir = root.join("project_rust").join(".git").join("objects");
    fs::create_dir_all(&git_dir).unwrap();
    fs::write(git_dir.join("dummy_obj"), vec![0u8; 512]).unwrap();

    let scanner = DevScanner::with_defaults();
    let report = scanner.scan_path(root);

    assert_eq!(report.items.len(), 2);

    let types: Vec<ArtifactType> = report.items.iter().map(|it| it.artifact_type).collect();
    assert!(types.contains(&ArtifactType::RustTarget));
    assert!(types.contains(&ArtifactType::NodeModules));

    assert!(report.total_bytes >= 1024);
}
