//! # dev-scrub
//!
//! Surgical Developer Disk Hygiene, Artifact Cleaner & Git-Aware Cache Vacuum in Rust.

pub mod scanner;
pub mod cleaner;
pub mod ui;

pub use scanner::{DevScanner, ScanReport, ArtifactItem, ArtifactType, GitActivity, GitInspector};
pub use cleaner::{VacuumEngine, CleanOptions, CleanResult};
pub use ui::TerminalHud;
