use dev_scrub::cleaner::VacuumEngine;
use std::path::Path;

#[test]
fn test_refuse_root_and_home() {
    assert!(VacuumEngine::is_safe_to_delete(Path::new("/")).is_err());
    assert!(VacuumEngine::is_safe_to_delete(Path::new("/root")).is_err());
    assert!(VacuumEngine::is_safe_to_delete(Path::new("/home")).is_err());
}

#[test]
fn test_refuse_non_artifacts() {
    // Arbitrary regular directories
    assert!(VacuumEngine::is_safe_to_delete(Path::new("/etc")).is_err());
    assert!(VacuumEngine::is_safe_to_delete(Path::new("/var")).is_err());
}
