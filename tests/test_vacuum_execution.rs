use dev_scrub::cleaner::{CleanOptions, VacuumEngine};
use dev_scrub::scanner::DevScanner;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_dry_run_vs_force_vacuum() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let target_dir = root.join("my_app").join("target");
    let src_dir = root.join("my_app").join("src");
    fs::create_dir_all(&target_dir).unwrap();
    fs::create_dir_all(&src_dir).unwrap();

    fs::write(target_dir.join("build.bin"), vec![1u8; 2048]).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    let scanner = DevScanner::with_defaults();
    let report = scanner.scan_path(root);
    assert_eq!(report.items.len(), 1);

    // 1. Dry Run Test: should simulate without deleting
    let dry_options = CleanOptions {
        dry_run: true,
        min_days_inactive: None,
        filter_types: None,
    };
    let dry_res = VacuumEngine::execute(&report.items, &dry_options);
    assert_eq!(dry_res.cleaned_items, 1);
    assert!(target_dir.exists(), "Dry run must NOT delete files on disk");

    // 2. Force Execution Test: should delete target/ but preserve src/main.rs!
    let force_options = CleanOptions {
        dry_run: false,
        min_days_inactive: None,
        filter_types: None,
    };
    let force_res = VacuumEngine::execute(&report.items, &force_options);
    assert_eq!(force_res.cleaned_items, 1);
    assert!(!target_dir.exists(), "Target directory must be deleted");
    assert!(
        src_dir.join("main.rs").exists(),
        "Source files MUST be preserved!"
    );
}
